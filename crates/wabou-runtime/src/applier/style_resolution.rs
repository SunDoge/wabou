use super::*;

pub(super) struct StyleState {
    pub(super) sheet: Option<StyleSheet>,
    pub(super) active_color_theme: Option<String>,
    pub(super) active_theme_colors: Arc<HashMap<String, u32>>,
    pub(super) theme: wabou_style::Theme,
    pub(super) rule_index: HashMap<Atom, Vec<usize>>,
    pub(super) universal_rules: Vec<usize>,
    pub(super) utility_cache: HashMap<Atom, Result<wabou_style::ParsedUtility, String>>,
    pub(super) class_resolution_cache: HashMap<Vec<Atom>, Arc<CachedClassResolution>>,
    #[cfg(test)]
    pub(super) class_resolution_cache_hits: usize,
    pub(super) warned_utility_classes: HashSet<Atom>,
    pub(super) warned_ir_properties: HashSet<Atom>,
    pub(super) inline_properties: HashMap<Atom, InlineProperty>,
    pub(super) diagnostics: HashMap<NodeId, Vec<String>>,
    #[cfg(any(feature = "devtools", test))]
    pub(super) cascade: HashMap<NodeId, Vec<StyleCascadeEntry>>,
}

impl Default for StyleState {
    fn default() -> Self {
        Self {
            sheet: None,
            active_color_theme: None,
            active_theme_colors: Arc::new(HashMap::new()),
            theme: wabou_style::Theme::default(),
            rule_index: HashMap::new(),
            universal_rules: Vec::new(),
            utility_cache: HashMap::new(),
            class_resolution_cache: HashMap::new(),
            #[cfg(test)]
            class_resolution_cache_hits: 0,
            warned_utility_classes: HashSet::new(),
            warned_ir_properties: HashSet::new(),
            inline_properties: HashMap::new(),
            diagnostics: HashMap::new(),
            #[cfg(any(feature = "devtools", test))]
            cascade: HashMap::new(),
        }
    }
}

struct ResolvedNodeStyle {
    layout: taffy::Style,
    paint: DeclaredPaint,
    host_text: Option<Arc<str>>,
    host_intrinsic: Option<[f32; 2]>,
    display_explicit: bool,
    diagnostics: Vec<String>,
    #[cfg(any(feature = "devtools", test))]
    cascade: Vec<StyleCascadeEntry>,
}

#[cfg(any(feature = "devtools", test))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StyleDeclarationSource {
    Universal,
    Class(Atom),
    Inline,
}

#[cfg(any(feature = "devtools", test))]
#[derive(Clone)]
pub(super) struct StyleCascadeEntry {
    pub(super) property: String,
    pub(super) source: StyleDeclarationSource,
    pub(super) overridden_sources: Vec<StyleDeclarationSource>,
}

#[cfg(any(feature = "devtools", test))]
fn record_style_source(
    cascade: &mut Vec<StyleCascadeEntry>,
    property: &str,
    source: StyleDeclarationSource,
) {
    if let Some(entry) = cascade.iter_mut().find(|entry| entry.property == property) {
        if entry.source != source {
            entry
                .overridden_sources
                .push(std::mem::replace(&mut entry.source, source));
        }
        return;
    }
    cascade.push(StyleCascadeEntry {
        property: property.to_owned(),
        source,
        overridden_sources: Vec::new(),
    });
}

fn resolve_color_tokens(value: &IrValue, colors: &HashMap<String, u32>) -> IrValue {
    match value {
        IrValue::Color {
            value: wabou_style::IrColor::Token { name },
        } => colors.get(name).map_or_else(
            || value.clone(),
            |rgba| IrValue::Color {
                value: wabou_style::IrColor::Literal { rgba: *rgba },
            },
        ),
        IrValue::List { values } => IrValue::List {
            values: values
                .iter()
                .map(|value| resolve_color_tokens(value, colors))
                .collect(),
        },
        IrValue::Record { fields } => IrValue::Record {
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), resolve_color_tokens(value, colors)))
                .collect(),
        },
        _ => value.clone(),
    }
}

impl Applier {
    pub(super) fn recompute_solid(&mut self, solid_id: NodeKey) {
        if let Some(&n) = self.document.node_store.solid_to_node.get(&solid_id) {
            self.recompute_node(n);
        }
    }

    pub(super) fn recompute_subtree(&mut self, node: NodeId) {
        self.recompute_node(node);
        let children = self
            .document
            .node_store
            .children
            .get(&node)
            .cloned()
            .unwrap_or_default();
        for child in children {
            self.recompute_subtree(child);
        }
    }

    /// Re-derive every node's `ComputedStyle` from the current `css` dict —
    /// called after a stylesheet host update.
    pub(super) fn recompute_all(&mut self) {
        let nodes: Vec<NodeId> = self
            .document
            .node_store
            .solid_to_node
            .values()
            .copied()
            .collect();
        for n in nodes {
            self.recompute_node(n);
        }
        self.rebuild_layout_boxes();
    }

    /// Re-resolve semantic colors without invalidating Taffy's retained
    /// layout. Color tokens can only target paint properties, so a palette
    /// animation must not turn every frame into a layout pass.
    pub(super) fn recompute_color_palette(&mut self) {
        let preserved = self.document.invalidation;
        let nodes: Vec<NodeId> = self
            .document
            .node_store
            .solid_to_node
            .values()
            .copied()
            .collect();
        for node in nodes {
            self.recompute_node(node);
        }
        self.inherit();
        self.document.invalidation.set(
            InvalidationFlags::LAYOUT,
            preserved.contains(InvalidationFlags::LAYOUT),
        );
        self.document.invalidation.set(
            InvalidationFlags::INHERIT,
            preserved.contains(InvalidationFlags::INHERIT),
        );
    }

    /// Facts for [`InlineFormattingContext::build`] from the retained tree.
    pub(super) fn node_facts(&self, node: NodeId) -> NodeFacts {
        let atoms = self.document.atoms.borrow();
        let decl = self.document.node_store.declared.get(&node);
        let tag = decl.and_then(|d| d.tag);
        let text = decl.and_then(|d| d.text.clone());
        let display = self
            .document
            .node_store
            .tree
            .style(node)
            .map(|s| s.display)
            .unwrap_or(taffy::Display::DEFAULT);
        let is_svg = tag.and_then(|tag| atoms.resolve(tag)) == Some("svg");
        let replaced = is_svg || self.document.widget_manager.widgets.contains_key(&node);
        NodeFacts {
            text_container: decl.is_some_and(|declared| {
                declared.text_behavior & crate::protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT != 0
            }),
            styled_text_container: decl.is_some_and(|declared| {
                declared.text_behavior & crate::protocol::TEXT_BEHAVIOR_AGGREGATE_STYLED != 0
            }),
            text: tag.is_none().then_some(text).flatten(),
            display,
            display_explicit: decl.is_some_and(|d| d.display_explicit),
            replaced,
        }
    }

    /// Project the logical tree into Taffy layout boxes via
    /// [`InlineFormattingContext`]. Only applies IFC output (children +
    /// collapsed text); does not re-implement formatting rules here.
    pub(super) fn rebuild_layout_boxes(&mut self) {
        #[cfg(test)]
        {
            self.document.ifc_projection_count += 1;
        }
        let ifc = InlineFormattingContext::build(&self.document.node_store.children, &|node| {
            self.node_facts(node)
        });

        let mut changed = Vec::new();
        self.document.node_store.inline_roots = ifc.roots;
        // Drop collapsed_text for parents that no longer collapse.
        let stale: Vec<NodeId> = self
            .document
            .node_store
            .collapsed_text
            .keys()
            .copied()
            .filter(|n| !ifc.collapsed_text.contains_key(n))
            .collect();
        for n in stale {
            self.document.node_store.collapsed_text.remove(&n);
            changed.push(n);
        }
        for (parent, text) in ifc.collapsed_text {
            if self.document.node_store.collapsed_text.get(&parent) != Some(&text) {
                self.document.node_store.collapsed_text.insert(parent, text);
                changed.push(parent);
            }
        }
        for (&parent, kids) in &self.document.node_store.children {
            let projected = if ifc.suppressed_children.contains(&parent) {
                &[]
            } else {
                kids.as_slice()
            };
            if self
                .document
                .node_store
                .tree
                .children(parent)
                .ok()
                .as_deref()
                != Some(projected)
            {
                let _ = self
                    .document
                    .node_store
                    .tree
                    .set_children(parent, projected);
            }
        }
        for node in changed {
            self.recompute_node_now(node);
        }
        self.document.ifc_dirty = false;
    }

    /// Propagate inherited text styles (`color`, `font-size`) top-down so a
    /// Resolve every node's [`DeclaredPaint`] against its parent into a
    /// fully computed [`Paint`] on the taffy node. Mirrors CSS inheritance
    /// without a full CSS engine. Run after `apply_frame` and before layout
    /// (the measure callback reads the effective `font_size`).
    pub(super) fn inherit(&mut self) {
        self.inherit_node(self.document.node_store.root, &InheritedPaint::default());
    }

    pub(super) fn serialize_svg(&self, root: NodeId, color: Color) -> Option<String> {
        fn escape_text(out: &mut String, value: &str) {
            for ch in value.chars() {
                match ch {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    _ => out.push(ch),
                }
            }
        }

        fn escape_attr(out: &mut String, value: &str, current_color: &str) {
            let value = value.replace("currentColor", current_color);
            for ch in value.chars() {
                match ch {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '"' => out.push_str("&quot;"),
                    '\'' => out.push_str("&apos;"),
                    _ => out.push(ch),
                }
            }
        }

        fn write_node(
            this: &Applier,
            atoms: &AtomPool,
            node: NodeId,
            current_color: &str,
            root: bool,
            out: &mut String,
        ) -> Option<()> {
            let decl = this.document.node_store.declared.get(&node)?;
            if let Some(text) = &decl.text {
                escape_text(out, text);
                return Some(());
            }
            let tag = atoms.resolve(decl.tag?)?;
            out.push('<');
            out.push_str(tag);
            let has_xmlns = decl
                .attrs
                .keys()
                .any(|name| atoms.resolve(*name) == Some("xmlns"));
            if root && !has_xmlns {
                out.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
            }
            for (name, value) in &decl.attrs {
                let Some(name) = atoms.resolve(*name) else {
                    continue;
                };
                // Solid's keyed reconciliation marker is not an SVG
                // presentation attribute and may contain arbitrary data.
                if name == "key" || name.starts_with("on") {
                    continue;
                }
                out.push(' ');
                out.push_str(name);
                out.push_str("=\"");
                escape_attr(out, value, current_color);
                out.push('"');
            }
            let children = this
                .document
                .node_store
                .children
                .get(&node)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if children.is_empty() {
                out.push_str("/>");
            } else {
                out.push('>');
                for child in children {
                    write_node(this, atoms, *child, current_color, false, out)?;
                }
                out.push_str("</");
                out.push_str(tag);
                out.push('>');
            }
            Some(())
        }

        let atoms = self.document.atoms.borrow();
        let decl = self.document.node_store.declared.get(&root)?;
        if decl.tag.and_then(|tag| atoms.resolve(tag)) != Some("svg") {
            return None;
        }
        let current_color = format!("{:x}", color.to_rgba8());
        if let Some(source) = decl.svg_source.as_ref() {
            return Some(source.replace("currentColor", &current_color));
        }
        let mut source = String::new();
        write_node(self, &atoms, root, &current_color, true, &mut source)?;
        Some(source)
    }

    pub(super) fn inherit_node(&mut self, node: NodeId, parent: &InheritedPaint) {
        let Some(decl) = self.document.node_store.declared.get(&node) else {
            return;
        };
        let declared = decl.paint.clone();
        let inherited = declared.resolve_inherited(parent);

        // Preserve host-owned content from the previous computed paint (text,
        // widget scene, intrinsic size). Cascade never owns these.
        let prev = self.document.node_store.tree.get_node_context(node);
        let mut host = HostPaint {
            text: prev.and_then(|p| p.text.clone()),
            text_runs: prev
                .map(|p| p.text_runs.clone())
                .unwrap_or_else(|| Arc::from([])),
            selection_rects: prev
                .map(|p| p.selection_rects.clone())
                .unwrap_or_else(|| Arc::from([])),
            text_max_lines: decl.text_max_lines,
            svg: None,
            vector_path: decl.vector_path.clone(),
            image: prev.and_then(|p| p.image.clone()),
            widget: prev.and_then(|p| p.widget.clone()),
            intrinsic_size: prev.and_then(|p| p.intrinsic_size),
            runtime_transform: self.document.runtime_transforms.get(&node).copied(),
            overlay_plane: self
                .document
                .overlay_planes
                .get(&node)
                .copied()
                .unwrap_or_default(),
            scrollbar: self
                .interaction
                .scroll
                .styles
                .get(&node)
                .copied()
                .unwrap_or_default(),
        };
        if let Some(source) = self.serialize_svg(node, inherited.text_color) {
            let cached_for_node = self
                .document
                .resources
                .svg
                .get(&node)
                .filter(|(cached_source, _)| cached_source.as_ref() == source)
                .map(|(_, image)| image.clone());
            host.svg = if let Some(image) = cached_for_node {
                Some(image)
            } else {
                let source: Arc<str> = Arc::from(source);
                let cached_asset = self
                    .document
                    .resources
                    .decoded_svg
                    .get(source.as_ref())
                    .cloned();
                let asset = cached_asset.unwrap_or_else(|| {
                    let parsed = legacy_shell::svg::SvgImage::parse(&source)
                        .map(Arc::new)
                        .map_err(|error| Arc::<str>::from(error.to_string()));
                    if let Err(error) = &parsed {
                        tracing::warn!(%error, "failed to parse inline SVG");
                    }
                    self.document
                        .resources
                        .decoded_svg
                        .insert(source.clone(), parsed.clone());
                    parsed
                });
                match asset {
                    Ok(image) => {
                        self.document
                            .resources
                            .svg
                            .insert(node, (source, image.clone()));
                        Some(image)
                    }
                    Err(_) => None,
                }
            };
        } else {
            self.document.resources.svg.remove(&node);
        }

        let paint = declared.resolve(parent, host);
        let _ = self
            .document
            .node_store
            .tree
            .set_node_context(node, Some(paint));

        let kids = self
            .document
            .node_store
            .children
            .get(&node)
            .cloned()
            .unwrap_or_default();
        for c in kids {
            self.inherit_node(c, &inherited);
        }

        if self.document.node_store.inline_roots.contains(&node) {
            let mut text = String::new();
            let mut runs = Vec::new();
            for child in self
                .document
                .node_store
                .children
                .get(&node)
                .cloned()
                .unwrap_or_default()
            {
                self.collect_styled_inline_runs(child, &mut text, &mut runs, 1.0);
            }
            if let Some(mut paint) = self
                .document
                .node_store
                .tree
                .get_node_context(node)
                .cloned()
            {
                paint.text = Some(Arc::from(text));
                paint.text_runs = Arc::from(runs);
                let _ = self
                    .document
                    .node_store
                    .tree
                    .set_node_context(node, Some(paint));
            }
        }
    }

    pub(super) fn collect_styled_inline_runs(
        &self,
        node: NodeId,
        text: &mut String,
        runs: &mut Vec<legacy_shell::text::TextRun>,
        ancestor_opacity: f32,
    ) {
        let Some(decl) = self.document.node_store.declared.get(&node) else {
            return;
        };
        let paint = self.document.node_store.tree.get_node_context(node);
        let opacity = ancestor_opacity * paint.map(|p| p.opacity).unwrap_or(1.0);
        if let Some(value) = &decl.text {
            let start = text.len();
            text.push_str(value);
            let end = text.len();
            if start != end {
                let mut color = legacy_shell::text::brush_for_color(
                    paint.map(|p| p.text_color).unwrap_or(Color::BLACK),
                );
                color[3] = ((color[3] as f32) * opacity.clamp(0.0, 1.0)).round() as u8;
                runs.push(legacy_shell::text::TextRun {
                    range: start..end,
                    font_size: paint.map(|p| p.font_size).unwrap_or(16.0),
                    font_weight: paint.map(|p| p.font_weight).unwrap_or(400.0),
                    font_italic: paint.is_some_and(|p| p.font_italic),
                    line_height: paint.and_then(|p| p.line_height),
                    color,
                });
            }
            return;
        }
        if let Some(children) = self.document.node_store.children.get(&node) {
            for child in children {
                self.collect_styled_inline_runs(*child, text, runs, opacity);
            }
        }
    }

    /// Re-derive `ComputedStyle` from declared state (classes via the css dict,
    /// inline applied on top so inline wins per-prop) and push to the taffy node.
    /// The root (#root, solid id 1) is skipped: its 100% viewport size + default
    /// Paint are host-provided and must not be overwritten by an empty
    /// `Declared` (which would reset the size to auto and collapse the tree).
    pub(super) fn recompute_node(&mut self, node: NodeId) {
        if self.document.applying_frame {
            self.document.dirty_styles.insert(node);
            return;
        }
        self.recompute_node_now(node);
    }

    /// Fast-path apply for a non-inherited inline style: update the cascaded
    /// [`DeclaredPaint`] + patch the matching non-inherited fields on the
    /// existing computed [`Paint`], skipping full class re-resolution and the
    /// O(N) inherit pass. Correct because a non-inherited inline property
    /// doesn't propagate to descendants. Hot path for animation (moving N
    /// nodes via top/left = 2N SetStyles/frame).
    pub(super) fn apply_inline_ir_fast(&mut self, node: NodeId, prop: &str, ir: &IrValue) -> bool {
        let Ok(existing) = self.document.node_store.tree.style(node) else {
            return false;
        };
        let Some(decl) = self.document.node_store.declared.get_mut(&node) else {
            return false;
        };
        let mut layout = existing.clone();
        let display_changed = prop == "display";
        if display_changed {
            decl.display_explicit = true;
        }
        if !style::apply_ir(&mut layout, &mut decl.paint, prop, ir) {
            return false;
        }
        #[cfg(any(feature = "devtools", test))]
        record_style_source(
            self.document.style.cascade.entry(node).or_default(),
            prop,
            StyleDeclarationSource::Inline,
        );
        self.document.ifc_dirty |= display_changed;
        let declared = decl.paint.clone();
        let layout_changed = existing != &layout;
        if layout_changed {
            let _ = self.document.node_store.tree.set_style(node, layout);
            self.document.invalidation.insert(InvalidationFlags::LAYOUT);
        }
        let transform_changed = matches!(
            prop,
            "transform"
                | "transform-translate-x"
                | "transform-translate-y"
                | "transform-scale"
                | "transform-rotate"
                | "transform-component"
        );
        if transform_changed || matches!(prop, "pointer-events" | "border-radius" | "z-index") {
            self.document
                .invalidation
                .insert(InvalidationFlags::GEOMETRY);
        }
        self.frame.projections.semantics_dirty |= transform_changed;
        // Patch only non-inherited computed fields; inherited fields stay at
        // their last resolved values (INHERIT is clear on this path).
        if let Some(mut paint) = self
            .document
            .node_store
            .tree
            .get_node_context(node)
            .cloned()
        {
            paint.background = declared.background;
            paint.opacity = declared.opacity;
            paint.transform = declared.transform;
            paint.shadows = declared.shadows;
            paint.border_radius = declared.border_radius;
            paint.border = declared.border;
            paint.outline_width = declared.outline_width;
            paint.outline_offset = declared.outline_offset;
            paint.outline_color = declared.outline_color;
            paint.text_ellipsis = declared.text_ellipsis;
            paint.pointer_events = declared.pointer_events;
            paint.z_index = declared.z_index;
            let _ = self
                .document
                .node_store
                .tree
                .set_node_context(node, Some(paint));
        }
        true
    }

    fn install_resolved_style(&mut self, node: NodeId, resolved: ResolvedNodeStyle) {
        let previous_layout = self.document.node_store.tree.style(node).ok().cloned();
        let projection_changed = previous_layout.as_ref().is_some_and(|previous| {
            previous.display != resolved.layout.display
                || self
                    .document
                    .node_store
                    .declared
                    .get(&node)
                    .is_some_and(|declared| declared.display_explicit != resolved.display_explicit)
        });
        self.document.ifc_dirty |= projection_changed;
        self.document
            .style
            .diagnostics
            .insert(node, resolved.diagnostics);
        #[cfg(any(feature = "devtools", test))]
        self.document.style.cascade.insert(node, resolved.cascade);
        if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
            declared.paint = resolved.paint.clone();
            declared.display_explicit = resolved.display_explicit;
        }
        let layout_style_changed = previous_layout.as_ref() != Some(&resolved.layout);
        if layout_style_changed {
            let _ = self
                .document
                .node_store
                .tree
                .set_style(node, resolved.layout);
        }

        // Resolve immediately against the parent so this node is current before
        // the next full inheritance walk updates its descendants.
        let parent = self
            .document
            .node_store
            .tree
            .parent(node)
            .and_then(|parent| self.document.node_store.tree.get_node_context(parent))
            .map(inherited_paint)
            .unwrap_or_default();
        let previous = self
            .document
            .node_store
            .tree
            .get_node_context(node)
            .cloned();
        let resource_image = self
            .document
            .node_store
            .declared
            .get(&node)
            .and_then(|declared| declared.image_resource)
            .and_then(|handle| self.document.resources.image_store.get(handle));
        let image = resource_image.as_ref().map(|resource| resource.drawable());
        let host = HostPaint {
            text: resolved.host_text,
            text_runs: previous
                .as_ref()
                .map(|paint| paint.text_runs.clone())
                .unwrap_or_else(|| Arc::from([])),
            selection_rects: previous
                .as_ref()
                .map(|paint| paint.selection_rects.clone())
                .unwrap_or_else(|| Arc::from([])),
            text_max_lines: self
                .document
                .node_store
                .declared
                .get(&node)
                .map_or(0, |declared| declared.text_max_lines),
            svg: previous.as_ref().and_then(|paint| paint.svg.clone()),
            vector_path: self
                .document
                .node_store
                .declared
                .get(&node)
                .and_then(|declared| declared.vector_path.clone()),
            image,
            widget: previous.as_ref().and_then(|paint| paint.widget.clone()),
            intrinsic_size: resolved.host_intrinsic,
            runtime_transform: self.document.runtime_transforms.get(&node).copied(),
            overlay_plane: self
                .document
                .overlay_planes
                .get(&node)
                .copied()
                .unwrap_or_default(),
            scrollbar: self
                .interaction
                .scroll
                .styles
                .get(&node)
                .copied()
                .unwrap_or_default(),
        };
        let paint = resolved.paint.resolve(&parent, host);
        let inherited_changed = previous
            .as_ref()
            .is_none_or(|previous| inherited_paint(previous) != inherited_paint(&paint));
        let text_layout_changed = previous.as_ref().is_none_or(|previous| {
            previous.text != paint.text
                || previous.text_runs != paint.text_runs
                || previous.intrinsic_size != paint.intrinsic_size
                || previous.font_size != paint.font_size
                || previous.font_weight != paint.font_weight
                || previous.line_height != paint.line_height
                || previous.wrap_text != paint.wrap_text
                || previous.text_ellipsis != paint.text_ellipsis
                || previous.text_max_lines != paint.text_max_lines
                || previous.text_align != paint.text_align
                || previous.font_family != paint.font_family
        });
        let geometry_changed = previous.as_ref().is_none_or(|previous| {
            previous.transform != paint.transform
                || previous.pointer_events != paint.pointer_events
                || previous.border_radius != paint.border_radius
                || previous.z_index != paint.z_index
        });
        let transform_changed = previous
            .as_ref()
            .is_none_or(|previous| previous.transform != paint.transform);
        let selection_policy_changed = previous.as_ref().is_none_or(|previous| {
            previous.text_selectable != paint.text_selectable
                || previous.text_select_all != paint.text_select_all
        });
        let _ = self
            .document
            .node_store
            .tree
            .set_node_context(node, Some(paint));
        if layout_style_changed || text_layout_changed {
            self.document.invalidation.insert(InvalidationFlags::LAYOUT);
        }
        if inherited_changed {
            self.document
                .invalidation
                .insert(InvalidationFlags::INHERIT);
        }
        if geometry_changed {
            self.document
                .invalidation
                .insert(InvalidationFlags::GEOMETRY);
        }
        self.frame.projections.semantics_dirty |= transform_changed || selection_policy_changed;
    }

    fn resolve_class_declarations(
        &mut self,
        class_key: Vec<Atom>,
        atoms: &AtomPool,
        active_theme_colors: &HashMap<String, u32>,
    ) -> Arc<CachedClassResolution> {
        if let Some(cached) = self.document.style.class_resolution_cache.get(&class_key) {
            #[cfg(feature = "profiling")]
            {
                self.frame.profile_class_cache_hits += 1;
            }
            #[cfg(test)]
            {
                self.document.style.class_resolution_cache_hits += 1;
            }
            return cached.clone();
        }
        #[cfg(feature = "profiling")]
        {
            self.frame.profile_class_cache_misses += 1;
        }

        let mut declarations = Vec::new();
        let mut diagnostics = Vec::new();
        if let Some(sheet) = &self.document.style.sheet {
            for &index in &self.document.style.universal_rules {
                let rule = &sheet.rules[index];
                for (index, declaration) in rule.declarations.iter().enumerate() {
                    declarations.push((
                        declaration.important,
                        rule.specificity,
                        0usize,
                        rule.source_order,
                        index,
                        declaration.property.clone(),
                        declaration.value.clone(),
                        None,
                    ));
                }
            }
            for (class_position, class) in class_key.iter().enumerate() {
                let Some(indices) = self.document.style.rule_index.get(class) else {
                    continue;
                };
                for &index in indices {
                    let rule = &sheet.rules[index];
                    for (index, declaration) in rule.declarations.iter().enumerate() {
                        declarations.push((
                            declaration.important,
                            rule.specificity,
                            class_position + 1,
                            rule.source_order,
                            index,
                            declaration.property.clone(),
                            declaration.value.clone(),
                            Some(*class),
                        ));
                    }
                }
            }
        }
        for (class_position, class) in class_key.iter().enumerate() {
            if self.document.style.rule_index.contains_key(class)
                || self.document.style.sheet.as_ref().is_some_and(|sheet| {
                    atoms
                        .resolve(*class)
                        .is_some_and(|name| sheet.ignores_class(name))
                })
            {
                continue;
            }
            let semantic_color = atoms
                .resolve(*class)
                .and_then(|name| {
                    ["bg-", "border-", "text-"]
                        .iter()
                        .find_map(|prefix| name.strip_prefix(prefix))
                })
                .filter(|token| active_theme_colors.contains_key(*token))
                .map(str::to_owned);
            let utility = self
                .document
                .style
                .utility_cache
                .entry(*class)
                .or_insert_with(|| {
                    atoms
                        .resolve(*class)
                        .ok_or_else(|| "unknown class atom".to_string())
                        .and_then(|name| {
                            wabou_style::parse_utility_with_theme(name, &self.document.style.theme)
                                .map_err(|error| error.to_string())
                        })
                });
            #[cfg(feature = "profiling")]
            {
                self.frame.profile_runtime_utility_fallbacks += 1;
            }
            let utility = match utility {
                Ok(utility) => utility,
                Err(diagnostic) => {
                    diagnostics.push(format!(
                        ".{}: {diagnostic}",
                        atoms.resolve(*class).unwrap_or("<unknown>")
                    ));
                    if self.document.style.warned_utility_classes.insert(*class) {
                        tracing::warn!(
                            class = atoms.resolve(*class).unwrap_or("<unknown>"),
                            %diagnostic,
                            "rejected runtime utility class"
                        );
                    }
                    continue;
                }
            };
            for (index, declaration) in utility.declarations.iter().enumerate() {
                let value = if let (Some(token), wabou_style::Value::Color { .. }) =
                    (semantic_color.as_ref(), &declaration.value)
                {
                    IrValue::Color {
                        value: wabou_style::IrColor::Token {
                            name: token.clone(),
                        },
                    }
                } else {
                    style_ir::utility_value(&declaration.value)
                };
                declarations.push((
                    false,
                    10,
                    class_position + 1,
                    0,
                    index,
                    declaration.property.clone(),
                    value,
                    Some(*class),
                ));
            }
        }
        declarations.sort_by_key(
            |(important, specificity, class_position, order, index, _, _, _)| {
                (*important, *specificity, *class_position, *order, *index)
            },
        );
        let cached = Arc::new(CachedClassResolution {
            declarations: declarations
                .into_iter()
                .map(
                    |(_, _, _, _, _, property, value, _source)| ResolvedClassDeclaration {
                        property,
                        value,
                        #[cfg(any(feature = "devtools", test))]
                        source: _source,
                    },
                )
                .collect(),
            diagnostics,
        });
        if self.document.style.class_resolution_cache.len() >= CLASS_RESOLUTION_CACHE_CAPACITY {
            self.document.style.class_resolution_cache.clear();
        }
        self.document
            .style
            .class_resolution_cache
            .insert(class_key, cached.clone());
        cached
    }

    pub(super) fn recompute_node_now(&mut self, node: NodeId) {
        if node == self.document.node_store.root {
            return;
        }
        let Some(class_key) = self
            .document
            .node_store
            .declared
            .get(&node)
            .map(|declared| declared.classes.clone())
        else {
            return;
        };
        let active_theme_colors = self.document.style.active_theme_colors.clone();
        let atoms_handle = self.document.atoms.clone();
        let atoms = atoms_handle.borrow();
        let cached =
            self.resolve_class_declarations(class_key, &atoms, active_theme_colors.as_ref());
        let decl = self
            .document
            .node_store
            .declared
            .get(&node)
            .expect("declared node remains present during style resolution");
        let resolved = {
            // Wabou has one tag-independent layout baseline. This is an
            // engine default, not an HTML-like tag stylesheet; authored
            // classes and inline style below may override it.
            let mut layout = taffy::Style {
                display: taffy::Display::Block,
                ..taffy::Style::default()
            };
            let mut paint = DeclaredPaint::default();
            let mut display_explicit = false;
            let mut diagnostics = Vec::new();
            #[cfg(any(feature = "devtools", test))]
            let mut cascade = Vec::new();
            // JS primitives author host defaults. Authored class and inline
            // declarations below may still opt into wrapping and shrinking.
            if decl.text_behavior & crate::protocol::TEXT_BEHAVIOR_SINGLE_LINE != 0 {
                layout.flex_shrink = 0.0;
                paint.wrap_text = Some(false);
            }
            diagnostics.extend(cached.diagnostics.iter().cloned());
            for declaration in &cached.declarations {
                let property = declaration.property.as_str();
                let value = &declaration.value;
                display_explicit |= property == "display";
                let value = resolve_color_tokens(value, &active_theme_colors);
                if style::apply_ir(&mut layout, &mut paint, property, &value) {
                    #[cfg(any(feature = "devtools", test))]
                    record_style_source(
                        &mut cascade,
                        property,
                        declaration.source.map_or(
                            StyleDeclarationSource::Universal,
                            StyleDeclarationSource::Class,
                        ),
                    );
                } else {
                    diagnostics.push(format!(
                        "{property}: unsupported Style IR property or value"
                    ));
                    if let Some(atom) = atoms.get(property)
                        && self.document.style.warned_ir_properties.insert(atom)
                    {
                        tracing::warn!(property, "unsupported Style IR property");
                    }
                }
            }
            for (property, value) in &decl.inline {
                if let Some(property) = atoms.resolve(*property) {
                    display_explicit |= property == "display";
                    let ir = resolve_color_tokens(&value.ir(), &active_theme_colors);
                    if style::apply_ir(&mut layout, &mut paint, property, &ir) {
                        #[cfg(any(feature = "devtools", test))]
                        record_style_source(&mut cascade, property, StyleDeclarationSource::Inline);
                    } else {
                        diagnostics
                            .push(format!("inline {property}: unsupported property or value"));
                    }
                }
            }
            // A growing native region is explicitly asking its parent for the
            // remaining width, so its contents must not silently impose CSS's
            // automatic min-content floor. Intrinsic/fixed controls keep the
            // Taffy default, while an authored min-width still wins.
            if layout.flex_grow > 0.0
                && layout.min_size.width == taffy::LengthPercentageAuto::auto()
            {
                layout.min_size.width = taffy::LengthPercentageAuto::length(0.0);
            }
            let mut host_intrinsic = None;
            if decl.tag.and_then(|tag| atoms.resolve(tag)) == Some("svg") {
                let view_box_size = decl
                    .attrs
                    .iter()
                    .find_map(|(name, value)| {
                        (atoms.resolve(*name) == Some("viewBox")).then(|| {
                            let values: Vec<f32> = value
                                .split_whitespace()
                                .filter_map(|part| part.parse().ok())
                                .collect();
                            (values.len() == 4 && values[2] > 0.0 && values[3] > 0.0)
                                .then_some([values[2], values[3]])
                        })
                    })
                    .flatten();
                host_intrinsic = Some(view_box_size.unwrap_or([300.0, 150.0]));
                // Width/height are SVG presentation attributes and provide the
                // replaced element's intrinsic CSS size. Utility/inline CSS
                // still wins when it supplied an explicit dimension.
                for (name, value) in &decl.attrs {
                    let Some(px) = value
                        .strip_suffix("px")
                        .unwrap_or(value)
                        .parse::<f32>()
                        .ok()
                    else {
                        continue;
                    };
                    match atoms.resolve(*name) {
                        Some("width") if layout.size.width.is_auto() => {
                            layout.size.width = taffy::Dimension::length(px);
                        }
                        Some("height") if layout.size.height.is_auto() => {
                            layout.size.height = taffy::Dimension::length(px);
                        }
                        _ => {}
                    }
                }
            }
            if decl.tag.and_then(|tag| atoms.resolve(tag)) == Some("img")
                && let Some(size) = decl
                    .image_resource
                    .and_then(|handle| self.document.resources.image_store.get(handle))
                    .map(|resource| {
                        let (width, height) = resource.dimensions();
                        [width as f32, height as f32]
                    })
            {
                host_intrinsic = Some(size);
            }
            // Replaced elements paint their own content. Standard text would
            // otherwise be drawn a second time underneath the widget scene.
            let host_text = (!self.document.widget_manager.widgets.contains_key(&node))
                .then(|| {
                    self.document
                        .node_store
                        .collapsed_text
                        .get(&node)
                        .cloned()
                        .or_else(|| decl.text.clone())
                })
                .flatten();
            if let Some(size) = self
                .document
                .widget_manager
                .widgets
                .get(&node)
                .and_then(|widget| widget.intrinsic_size())
            {
                host_intrinsic = Some(size);
            }
            ResolvedNodeStyle {
                layout,
                paint,
                host_text,
                host_intrinsic,
                display_explicit,
                diagnostics,
                #[cfg(any(feature = "devtools", test))]
                cascade: {
                    cascade.sort_by(|left, right| left.property.cmp(&right.property));
                    cascade
                },
            }
        };
        self.install_resolved_style(node, resolved);
    }
}

fn inherited_paint(paint: &Paint) -> InheritedPaint {
    InheritedPaint {
        cursor: paint.cursor,
        text_color: paint.text_color,
        font_size: paint.font_size,
        font_weight: paint.font_weight,
        font_italic: paint.font_italic,
        letter_spacing: paint.letter_spacing,
        line_height: paint.line_height,
        wrap_text: paint.wrap_text,
        text_selectable: paint.text_selectable,
        text_select_all: paint.text_select_all,
        text_align: paint.text_align,
        font_family: paint.font_family.clone(),
    }
}
