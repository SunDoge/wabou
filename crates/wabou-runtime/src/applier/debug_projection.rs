use super::style_resolution::{StyleCascadeEntry, StyleDeclarationSource};
use super::*;

fn debug_rect([x0, y0, x1, y1]: [f32; 4]) -> wabou_devtools::Rect {
    wabou_devtools::Rect {
        x: x0,
        y: y0,
        width: (x1 - x0).max(0.0),
        height: (y1 - y0).max(0.0),
    }
}

fn intersect_rect(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    [
        left[0].max(right[0]),
        left[1].max(right[1]),
        left[2].min(right[2]),
        left[3].min(right[3]),
    ]
}

fn debug_attrs(declared: Option<&Declared>, atoms: &AtomPool) -> Vec<(String, String)> {
    let mut attrs: Vec<_> = declared
        .into_iter()
        .flat_map(|declared| declared.attrs.iter())
        .filter_map(|(name, value)| {
            atoms.resolve(*name).map(|name| {
                let lower = name.to_ascii_lowercase();
                let value = if ["password", "token", "secret", "authorization"]
                    .iter()
                    .any(|needle| lower.contains(needle))
                {
                    "[REDACTED]".to_owned()
                } else {
                    value.chars().take(4096).collect()
                };
                (name.to_owned(), value)
            })
        })
        .collect();
    attrs.sort_by(|left, right| left.0.cmp(&right.0));
    attrs
}

fn debug_classes(declared: Option<&Declared>, atoms: &AtomPool) -> Vec<String> {
    declared
        .into_iter()
        .flat_map(|declared| declared.classes.iter())
        .filter_map(|class| atoms.resolve(*class).map(str::to_owned))
        .collect()
}

fn debug_style_source(source: StyleDeclarationSource, atoms: &AtomPool) -> String {
    match source {
        StyleDeclarationSource::Universal => "*".to_owned(),
        StyleDeclarationSource::Class(class) => {
            format!(".{}", atoms.resolve(class).unwrap_or("<unknown>"))
        }
        StyleDeclarationSource::Inline => "inline".to_owned(),
    }
}

fn debug_style_cascade(
    entries: Option<&Vec<StyleCascadeEntry>>,
    atoms: &AtomPool,
) -> Vec<wabou_devtools::DebugStyleCascade> {
    entries
        .into_iter()
        .flatten()
        .map(|entry| wabou_devtools::DebugStyleCascade {
            property: entry.property.clone(),
            source: debug_style_source(entry.source, atoms),
            overridden_sources: entry
                .overridden_sources
                .iter()
                .map(|source| debug_style_source(*source, atoms))
                .collect(),
        })
        .collect()
}

fn resolved_css_transforms(placed: &[PlacedNode]) -> HashMap<NodeId, Affine> {
    let mut transforms = HashMap::with_capacity(placed.len());
    for node in placed {
        let parent = node
            .parent_node_id
            .and_then(|parent| transforms.get(&parent).copied())
            .unwrap_or(Affine::IDENTITY);
        transforms.insert(
            node.node_id,
            wabou_shell::scene::resolve_node_transform(node, parent),
        );
    }
    transforms
}

impl Applier {
    fn debug_clip_info(
        &self,
        placed_node: &PlacedNode,
        id: NodeKey,
        placed_by_id: &HashMap<NodeId, &PlacedNode>,
        css_transforms: &HashMap<NodeId, Affine>,
    ) -> wabou_devtools::DebugClipInfo {
        let [cx, cy] = placed_node.content_origin;
        let [cw, ch] = placed_node.content_size;
        let border_transform = css_transforms[&placed_node.node_id];
        let content_transform = border_transform * Affine::translate((cx as f64, cy as f64));
        let (static_transform, _) = wabou_shell::scene::resolve_local_transforms(placed_node);
        let mut chain = Vec::new();
        let mut ancestor_id = placed_node.parent_node_id;
        while let Some(node_id) = ancestor_id {
            let Some(ancestor) = placed_by_id.get(&node_id).copied() else {
                break;
            };
            if let Some(rect) = ancestor.own_clip {
                chain.push(wabou_devtools::DebugClip {
                    node_id: self
                        .document
                        .node_store
                        .node_to_solid
                        .get(&node_id)
                        .copied()
                        .unwrap_or(NodeKey::ROOT),
                    kind: "ancestor-overflow".into(),
                    coordinate_space: "layout-window-logical".into(),
                    rect: debug_rect(rect),
                    radius: ancestor.own_clip_radius,
                    transform: css_transforms[&node_id].as_coeffs(),
                });
            }
            ancestor_id = ancestor.parent_node_id;
        }
        chain.reverse();
        if let Some(rect) = placed_node.own_clip {
            chain.push(wabou_devtools::DebugClip {
                node_id: id,
                kind: "self-overflow".into(),
                coordinate_space: "layout-window-logical".into(),
                rect: debug_rect(rect),
                radius: placed_node.own_clip_radius,
                transform: border_transform.as_coeffs(),
            });
        }
        let widget_local = self
            .document
            .widget_manager
            .widgets
            .contains_key(&placed_node.node_id)
            .then(|| {
                let border_inset = placed_node.border_widths.into_iter().fold(0.0, f32::max);
                wabou_devtools::DebugClip {
                    node_id: id,
                    kind: "widget-content".into(),
                    coordinate_space: "content-local".into(),
                    rect: wabou_devtools::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: cw,
                        height: ch,
                    },
                    radius: (placed_node.paint.border_radius - border_inset).max(0.0),
                    transform: content_transform.as_coeffs(),
                }
            });
        let widget_self_clip = widget_local.as_ref().and_then(|clip| {
            if clip.radius > 0.0 {
                Some(([cx, cy, cx + cw, cy + ch], clip.radius))
            } else {
                placed_node
                    .own_clip
                    .map(|rect| (rect, placed_node.own_clip_radius))
            }
        });
        let effective_rect = match (placed_node.clip, widget_self_clip) {
            (Some(inherited), Some((local, _))) => Some((intersect_rect(inherited, local), 0.0)),
            (Some(inherited), None) => Some((inherited, placed_node.clip_radius)),
            (None, Some(local)) => Some(local),
            (None, None) => None,
        };
        let axis_aligned = chain
            .iter()
            .all(|clip| clip.transform == Affine::IDENTITY.as_coeffs())
            && (widget_self_clip.is_none() || border_transform == Affine::IDENTITY);
        let effective = axis_aligned
            .then_some(effective_rect)
            .flatten()
            .map(|(rect, radius)| wabou_devtools::DebugClip {
                node_id: id,
                kind: "effective".into(),
                coordinate_space: "window-logical".into(),
                rect: debug_rect(rect),
                radius,
                transform: Affine::IDENTITY.as_coeffs(),
            });
        wabou_devtools::DebugClipInfo {
            widget_local,
            chain,
            effective,
            static_transform: static_transform.as_coeffs(),
            runtime_transform: placed_node
                .paint
                .runtime_transform
                .map(|matrix| matrix.map(f64::from)),
            border_transform: border_transform.as_coeffs(),
            scene_transform: content_transform.as_coeffs(),
            device_scale: self.frame.device_scale,
        }
    }

    fn debug_listeners(&self, id: NodeKey) -> Vec<u8> {
        let mut listeners: Vec<_> = self
            .interaction
            .input
            .listeners
            .get(&id)
            .into_iter()
            .flat_map(|events| events.codes())
            .collect();
        listeners.sort_unstable();
        listeners
    }

    fn debug_matched_rules(&self, declared: Option<&Declared>) -> Vec<String> {
        declared
            .into_iter()
            .flat_map(|declared| {
                std::iter::once(&self.document.style.universal_rules).chain(
                    declared
                        .classes
                        .iter()
                        .filter_map(|class| self.document.style.rule_index.get(class)),
                )
            })
            .flatten()
            .filter_map(|index| self.document.style.sheet.as_ref()?.rules.get(*index))
            .map(|rule| {
                if rule.class_name == "*" {
                    "*".to_owned()
                } else {
                    format!(".{}", rule.class_name)
                }
            })
            .collect()
    }

    pub(super) fn publish_debug_snapshot(
        &mut self,
        placed: &[PlacedNode],
        text_context: &mut TextContext,
    ) {
        let Some(state) = self.frame.projections.debug_state.clone() else {
            return;
        };
        self.frame.projections.debug_revision =
            self.frame.projections.debug_revision.wrapping_add(1);
        let atoms = self.document.atoms.borrow();
        let placed_by_id: HashMap<_, _> = placed.iter().map(|node| (node.node_id, node)).collect();
        let css_transforms = resolved_css_transforms(placed);
        let semantic_snapshot = &self.frame.projections.semantic_snapshot;
        let semantic_by_id: HashMap<_, _> = semantic_snapshot
            .nodes
            .iter()
            .map(|node| (NodeKey::from_ffi(node.id), node))
            .collect();
        let exposed_semantics: HashSet<_> = semantic_snapshot
            .exposed_nodes()
            .into_iter()
            .map(|node| NodeKey::from_ffi(node.id))
            .collect();
        let mut nodes = Vec::with_capacity(placed.len());
        for placed_node in placed {
            let Some(&id) = self
                .document
                .node_store
                .node_to_solid
                .get(&placed_node.node_id)
            else {
                continue;
            };
            let declared = self.document.node_store.declared.get(&placed_node.node_id);
            let tag = declared
                .and_then(|declared| declared.tag)
                .and_then(|tag| atoms.resolve(tag))
                .unwrap_or(if id == NodeKey::ROOT {
                    "#root"
                } else {
                    "#text"
                })
                .to_owned();
            let attrs = debug_attrs(declared, &atoms);
            let listeners = self.debug_listeners(id);
            let classes = debug_classes(declared, &atoms);
            let matched_rules = self.debug_matched_rules(declared);
            let [x0, y0, x1, y1] = placed_node.rect;
            let [cx, cy] = placed_node.content_origin;
            let [cw, ch] = placed_node.content_size;
            let layout = self
                .document
                .node_store
                .tree
                .style(placed_node.node_id)
                .ok();
            let clip = self.debug_clip_info(placed_node, id, &placed_by_id, &css_transforms);
            let text_layout = wabou_shell::scene::layout_node_text(text_context, placed_node);
            let synthesis = text_layout
                .as_deref()
                .map(wabou_shell::text::text_synthesis)
                .unwrap_or_default();
            let text_metrics = text_layout
                .as_deref()
                .and_then(|layout| {
                    (layout.lines().len() == 1)
                        .then(|| {
                            wabou_shell::text::single_line_text_metrics(layout, layout.height())
                        })
                        .flatten()
                        .map(|metrics| ("node", metrics))
                })
                .or_else(|| {
                    self.document
                        .widget_manager
                        .widgets
                        .get(&placed_node.node_id)
                        .and_then(|widget| widget.text_metrics())
                        .map(|metrics| ("widget", metrics))
                })
                .map(|(source, metrics)| wabou_devtools::DebugTextMetrics {
                    source: source.to_owned(),
                    line_box: wabou_devtools::Rect {
                        x: cx + metrics.line_box[0],
                        y: cy + metrics.line_box[1],
                        width: metrics.line_box[2],
                        height: metrics.line_box[3],
                    },
                    baseline: cy + metrics.baseline,
                });
            nodes.push(wabou_devtools::DebugNode {
                id,
                parent_id: placed_node.parent_node_id.and_then(|parent| {
                    self.document.node_store.node_to_solid.get(&parent).copied()
                }),
                tag,
                text: placed_node
                    .paint
                    .text
                    .as_deref()
                    .map(|text| text.chars().take(4096).collect()),
                text_metrics,
                classes,
                matched_rules,
                style_diagnostics: self
                    .document
                    .style
                    .diagnostics
                    .get(&placed_node.node_id)
                    .cloned()
                    .unwrap_or_default(),
                style_cascade: debug_style_cascade(
                    self.document.style.cascade.get(&placed_node.node_id),
                    &atoms,
                ),
                attrs,
                rect: wabou_devtools::Rect {
                    x: x0,
                    y: y0,
                    width: x1 - x0,
                    height: y1 - y0,
                },
                content_rect: wabou_devtools::Rect {
                    x: cx,
                    y: cy,
                    width: cw,
                    height: ch,
                },
                listeners,
                focusable: self.interaction.input.focusable_targets.contains(&id),
                focus_order: declared.and_then(|declared| declared.focus_order),
                semantic: semantic_by_id.get(&id).map(|semantic| {
                    wabou_devtools::DebugSemanticProjection {
                        role: semantic.role.as_str().to_owned(),
                        label: semantic.label.clone(),
                        disabled: semantic.disabled,
                        exposed: exposed_semantics.contains(&id),
                        controls: semantic
                            .controls
                            .iter()
                            .copied()
                            .map(NodeKey::from_ffi)
                            .collect(),
                        active_descendant: semantic.active_descendant.map(NodeKey::from_ffi),
                        states: wabou_devtools::DebugSemanticStates {
                            checked: semantic
                                .states
                                .checked
                                .map(|state| state.as_str().to_owned()),
                            pressed: semantic
                                .states
                                .pressed
                                .map(|state| state.as_str().to_owned()),
                            selected: semantic.states.selected,
                            expanded: semantic.states.expanded,
                            current: semantic
                                .states
                                .current
                                .map(|state| state.as_str().to_owned()),
                            popup: semantic.states.popup.map(|state| state.as_str().to_owned()),
                            modal: semantic.states.modal,
                        },
                        range: wabou_devtools::DebugSemanticRange {
                            value: semantic.numeric_value,
                            min: semantic.min_numeric_value,
                            max: semantic.max_numeric_value,
                        },
                    }
                }),
                widget: self
                    .document
                    .widget_manager
                    .widgets
                    .contains_key(&placed_node.node_id)
                    .then(|| "native".into()),
                clip,
                computed: wabou_devtools::DebugComputedStyle {
                    display: layout.map(|style| format!("{:?}", style.display)),
                    position: layout.map(|style| format!("{:?}", style.position)),
                    overflow_x: layout.map(|style| format!("{:?}", style.overflow.x)),
                    overflow_y: layout.map(|style| format!("{:?}", style.overflow.y)),
                    font_size: placed_node.paint.font_size,
                    font_weight: placed_node.paint.font_weight,
                    font_italic: placed_node.paint.font_italic,
                    font_family: placed_node.paint.font_family.as_deref().map(str::to_owned),
                    synthetic_bold: synthesis.embolden,
                    synthetic_italic: synthesis.skew,
                    wrap_text: placed_node.paint.wrap_text,
                    opacity: placed_node.paint.opacity,
                    pointer_events: placed_node.paint.pointer_events,
                    z_index: placed_node.paint.z_index,
                    overlay_plane: format!("{:?}", placed_node.paint.overlay_plane),
                    scrollbar_opacity: placed_node.scroll.opacity,
                    text_color: format!("{:x}", placed_node.paint.text_color.to_rgba8()),
                    background: placed_node
                        .paint
                        .background
                        .map(|color| format!("{:x}", color.to_rgba8())),
                },
            });
        }
        let snapshot = wabou_devtools::DebugSnapshot {
            status: wabou_devtools::DebugStatus {
                protocol_version: wabou_devtools::PROTOCOL_VERSION,
                pid: std::process::id(),
                revision: self.frame.projections.debug_revision,
                viewport_width: self.frame.last_viewport.0,
                viewport_height: self.frame.last_viewport.1,
                device_scale: self.frame.device_scale,
                node_count: nodes.len(),
                text_backend: text_context.raster_backend_name().to_owned(),
                text_outline_fallback: text_context.outline_fallback_name().to_owned(),
                focused_node: self.interaction.input.focused_target,
                hovered_node: self.interaction.input.hovered_target,
                overlay: Default::default(),
                overlay_paint: Default::default(),
            },
            nodes,
        };
        drop(atoms);
        if let Ok(mut state) = state.write() {
            state.publish(snapshot);
        }
    }
}
