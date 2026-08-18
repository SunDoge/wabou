use super::*;

fn style_value_ir(value: crate::protocol::StyleValue) -> IrValue {
    use crate::protocol::StyleValue;
    use wabou_shell::style::IrLength;

    match value {
        StyleValue::Px(value) => IrValue::Length {
            value: IrLength::Px { value },
        },
        StyleValue::Percent(value) => IrValue::Length {
            value: IrLength::Percent { value },
        },
        StyleValue::Number(value) => IrValue::Number { value },
        StyleValue::Boolean(value) => IrValue::Boolean { value },
        StyleValue::Color(rgba) => IrValue::Color {
            value: wabou_shell::style::IrColor::Literal { rgba },
        },
        StyleValue::Auto => IrValue::Length {
            value: IrLength::Auto,
        },
    }
}

impl Applier {
    fn inline_property(&mut self, atom: Atom) -> Option<InlineProperty> {
        if let Some(property) = self.inline_properties.get(&atom) {
            return Some(property.clone());
        }
        let name: Arc<str> = {
            let atoms = self.atoms.borrow();
            Arc::from(atoms.resolve(atom)?)
        };
        let inherited = INHERITED_PROPERTIES.contains(&name.as_ref()) || name.as_ref() == "font";
        let property = InlineProperty { name, inherited };
        self.inline_properties.insert(atom, property.clone());
        Some(property)
    }

    fn set_inline_ir(&mut self, id: u32, prop: Atom, ir: IrValue) {
        let Some(&node) = self.node_store.solid_to_node.get(&id) else {
            return;
        };
        let Some(property) = self.inline_property(prop) else {
            tracing::warn!(atom = prop.get(), "unknown style-property atom");
            return;
        };
        if let Some(declared) = self.node_store.declared.get_mut(&node) {
            declared.inline.insert(prop, InlineValue::Typed(ir.clone()));
        }

        // Non-inherited values can update the post-inheritance computed style
        // directly. Inherited values must propagate through the subtree.
        if property.inherited {
            self.recompute_node(node);
        } else if !self.apply_inline_ir_fast(node, &property.name, &ir) {
            if let Some(declared) = self.node_store.declared.get_mut(&node) {
                declared.inline.remove(&prop);
            }
            if self.warned_ir_properties.insert(prop) {
                tracing::warn!(property = %property.name, "unsupported inline style property or value");
            }
        }
    }

    fn create_element(&mut self, id: u32, tag: Atom, attrs: &[(Atom, &str)]) {
        let mut declared = Declared {
            tag: Some(tag),
            ..Declared::default()
        };
        let class_value = {
            let atoms = self.atoms.borrow();
            if atoms.resolve(tag).is_none() {
                tracing::warn!(atom = tag.get(), "unknown tag atom");
            }
            attrs.iter().find_map(|(name, value)| {
                matches!(atoms.resolve(*name), Some("class" | "className")).then_some(*value)
            })
        };
        if let Some(value) = class_value {
            // CreateElement attributes are retained for protocol compatibility;
            // class tokens normally arrive as atoms through SetClassName.
            let mut atoms = self.atoms.borrow_mut();
            declared.classes = value
                .split_whitespace()
                .map(|value| atoms.intern(value))
                .collect();
        }
        declared
            .attrs
            .extend(attrs.iter().map(|(name, value)| (*name, Arc::from(*value))));

        let node = self.node_store.create_leaf(id, declared);
        self.recompute_solid(id);
        let Some(mut widget) = self.widget_manager.create(tag, self.wake_callback.as_ref()) else {
            return;
        };
        let atoms = self.atoms.borrow();
        for (name, value) in attrs {
            if let Some(name) = atoms.resolve(*name) {
                widget.attribute_changed(name, value);
            }
        }
        drop(atoms);
        let mounted_changes = widget.mounted();
        self.widget_manager.widgets.insert(node, widget);
        self.invalidate_widget_changes(mounted_changes);
        self.drain_widget_host_actions(node);
        self.drain_widget_node_events(node);
        self.recompute_node(node);
    }

    fn set_shadows(&mut self, id: u32, shadows: &[crate::protocol::ShadowValue]) {
        let Some(&node) = self.node_store.solid_to_node.get(&id) else {
            return;
        };
        let prop = self.atoms.borrow_mut().intern("box-shadow");
        let values = shadows
            .iter()
            .map(|shadow| {
                let length = |value| IrValue::Length {
                    value: wabou_shell::style::IrLength::Px { value },
                };
                let mut fields = HashMap::from([
                    ("x".to_owned(), length(shadow.offset_x)),
                    ("y".to_owned(), length(shadow.offset_y)),
                    ("spread".to_owned(), length(shadow.spread)),
                    ("stdDev".to_owned(), length(shadow.std_dev)),
                    (
                        "color".to_owned(),
                        IrValue::Color {
                            value: wabou_shell::style::IrColor::Literal { rgba: shadow.color },
                        },
                    ),
                ]);
                if let Some(radius) = shadow.radius {
                    fields.insert("radius".to_owned(), length(radius));
                }
                IrValue::Record { fields }
            })
            .collect();
        let ir = IrValue::List { values };
        if let Some(declared) = self.node_store.declared.get_mut(&node) {
            declared.inline.insert(prop, InlineValue::Typed(ir.clone()));
        }
        if !self.apply_inline_ir_fast(node, "box-shadow", &ir) {
            if let Some(declared) = self.node_store.declared.get_mut(&node) {
                declared.inline.remove(&prop);
            }
            tracing::warn!("invalid Vello shadow list");
        }
    }

    fn drop_node(&mut self, id: u32) {
        if self.input.pointer_down_target == Some(id) {
            self.cancel_active_pointer_gesture();
        }
        let node = self.node_store.solid_to_node.get(&id).copied();
        if self
            .text_selection
            .active
            .as_ref()
            .is_some_and(|active| active.anchor_target == id || active.focus_target == id)
        {
            self.text_selection.active = None;
            self.text_selection.next_scroll = None;
            self.sync_text_selection_change();
        }
        if self.input.focused_target == Some(id) {
            if self.input.window_focused
                && let Some(widget) =
                    node.and_then(|node| self.widget_manager.widgets.get_mut(&node))
            {
                widget.focus_changed(false);
            }
            self.input.focused_target = None;
        }
        self.input.listeners.remove(&id);
        self.scroll.pending_events.remove(&id);
        self.resize_targets.borrow_mut().remove(&id);
        // Never retain an id after its generational node has been removed.
        if self.input.hovered_target == Some(id) {
            self.input.hovered_target = None;
        }

        let Some(node) = self.node_store.remove(id) else {
            return;
        };
        self.clear_image_source(node);
        self.runtime_transforms.remove(&node);
        self.overlay_planes.remove(&node);
        self.scroll.styles.remove(&node);
        self.scroll.offsets.remove(&node);
        self.resources.svg.remove(&node);
        self.style_diagnostics.remove(&node);
        if let Some(widget) = self.widget_manager.widgets.get_mut(&node) {
            widget.unmount();
        }
        self.drain_widget_host_actions(node);
        self.widget_manager.widgets.remove(&node);
        self.widget_manager.styles.remove(&node);
        self.widget_manager.geometries.remove(&node);
        self.widget_manager.visibility.remove(&node);
        self.widget_manager
            .host_action_routes
            .retain(|_, (widget_node, _)| *widget_node != node);
        self.invalidation.insert(InvalidationFlags::LAYOUT);
    }

    fn set_attribute(&mut self, id: u32, name: Atom, value: &str) {
        let Some(&node) = self.node_store.solid_to_node.get(&id) else {
            return;
        };
        let is_class = matches!(
            self.atoms.borrow().resolve(name),
            Some("class" | "className")
        );
        if let Some(declared) = self.node_store.declared.get_mut(&node) {
            if is_class {
                let mut atoms = self.atoms.borrow_mut();
                declared.classes = value
                    .split_whitespace()
                    .map(|value| atoms.intern(value))
                    .collect();
            }
            declared.attrs.insert(name, Arc::from(value));
        }
        if self.atoms.borrow().resolve(name) == Some("image-source") {
            if let Some(url) = remote_image_url(value) {
                self.load_image_source(node, &url);
            } else {
                self.clear_image_source(node);
            }
        }
        let widget_changes = if !is_class
            && let Some(widget) = self.widget_manager.widgets.get_mut(&node)
            && let Some(name) = self.atoms.borrow().resolve(name)
        {
            widget.attribute_changed(name, value)
        } else {
            wabou_shell::WidgetChanges::empty()
        };
        self.invalidate_widget_changes(widget_changes);
        self.recompute_node(node);
    }

    fn set_widget_config(&mut self, id: u32, json: &str) {
        let Some(&node) = self.node_store.solid_to_node.get(&id) else {
            return;
        };
        if let Some(widget) = self.widget_manager.widgets.get_mut(&node) {
            match widget.config_changed(json) {
                Ok(changes) => self.invalidate_widget_changes(changes),
                Err(error) => {
                    tracing::warn!(solid_id = id, %error, "widget rejected widgetConfig");
                }
            }
        }
    }

    fn remove_widget_config(&mut self, id: u32) {
        let Some(&node) = self.node_store.solid_to_node.get(&id) else {
            return;
        };
        if let Some(widget) = self.widget_manager.widgets.get_mut(&node) {
            let changes = widget.config_removed();
            self.invalidate_widget_changes(changes);
        }
    }

    fn load_image_source(&mut self, node: NodeId, value: &str) {
        let source: Arc<str> = Arc::from(value);
        self.clear_image_source(node);
        self.resources
            .node_image_sources
            .insert(node, source.clone());
        if let Some(result) = self.resources.cache.raster(source.as_ref()) {
            self.dispatch_image_resource_result(node, source.as_ref(), &result);
            return;
        }
        self.resources
            .image_subscribers
            .entry(source.clone())
            .or_default()
            .insert(node);
        if !self.resources.pending_images.insert(source.clone()) {
            return;
        }
        let Ok(url) = url::Url::parse(value) else {
            self.resources.pending_images.remove(&source);
            let result = Err(Arc::from("network image URL must use HTTP(S)"));
            self.resources
                .cache
                .insert_raster(source.to_string(), result.clone());
            self.finish_image_source(&source, &result);
            return;
        };
        if !matches!(url.scheme(), "http" | "https") {
            self.resources.pending_images.remove(&source);
            let result = Err(Arc::from("network image URL must use HTTP(S)"));
            self.resources
                .cache
                .insert_raster(source.to_string(), result.clone());
            self.finish_image_source(&source, &result);
            return;
        }
        let tx = self.resources.result_tx.clone();
        let wake = self.wake_callback.clone();
        let asset_cache = self.resources.cache.clone();
        tracing::debug!(source = %source, "loading network image");
        let load = async move {
            let result = async {
                let bytes = asset_cache.network_image_bytes(url).await?;
                wabou_shell::image::RasterImage::decode(&bytes)
                    .map(Arc::new)
                    .map_err(|error| error.to_string())
            }
            .await
            .map_err(Arc::<str>::from);
            let _ = tx.send(ImageLoadResult { source, result });
            if let Some(wake) = wake {
                wake();
            }
        };
        self.resources.cache.spawn(load);
    }

    pub(super) fn finish_image_source(
        &mut self,
        source: &Arc<str>,
        result: &crate::asset_cache::RasterAsset,
    ) {
        let nodes = self
            .resources
            .image_subscribers
            .remove(source)
            .unwrap_or_default();
        for node in nodes {
            if self.resources.node_image_sources.get(&node) == Some(source) {
                self.recompute_node(node);
                self.dispatch_image_resource_result(node, source, result);
            }
        }
    }

    pub(super) fn clear_image_source(&mut self, node: NodeId) {
        let Some(source) = self.resources.node_image_sources.remove(&node) else {
            return;
        };
        if let Some(nodes) = self.resources.image_subscribers.get_mut(&source) {
            nodes.remove(&node);
            if nodes.is_empty() {
                self.resources.image_subscribers.remove(&source);
            }
        }
    }

    fn remove_attribute(&mut self, id: u32, name: Atom) {
        let Some(&node) = self.node_store.solid_to_node.get(&id) else {
            return;
        };
        let is_class = matches!(
            self.atoms.borrow().resolve(name),
            Some("class" | "className")
        );
        if let Some(declared) = self.node_store.declared.get_mut(&node) {
            declared.attrs.remove(&name);
            if is_class {
                declared.classes.clear();
            }
        }
        if self.atoms.borrow().resolve(name) == Some("image-source") {
            self.clear_image_source(node);
        }
        let widget_changes = if let Some(widget) = self.widget_manager.widgets.get_mut(&node)
            && let Some(name) = self.atoms.borrow().resolve(name)
        {
            widget.attribute_removed(name)
        } else {
            wabou_shell::WidgetChanges::empty()
        };
        self.invalidate_widget_changes(widget_changes);
        self.recompute_node(node);
    }

    /// Decode + apply one frame's ops in order.
    pub(super) fn apply_frame(&mut self, frame: &Frame) {
        self.batching_styles = true;
        for op in &frame.ops {
            self.apply_op(op);
        }
        self.batching_styles = false;
        let dirty = std::mem::take(&mut self.dirty_styles);
        for node in dirty {
            self.recompute_node_now(node);
        }
        self.rebuild_layout_boxes();
    }

    pub(super) fn apply_op(&mut self, op: &Op) {
        match op {
            Op::SetTransform2D { id, .. } => {
                let affects_hit_geometry =
                    self.node_store.solid_to_node.get(id).is_some_and(|node| {
                        self.node_store
                            .tree
                            .get_node_context(*node)
                            .is_some_and(|paint| paint.pointer_events)
                            || self
                                .node_store
                                .children
                                .get(node)
                                .is_some_and(|children| !children.is_empty())
                    });
                if affects_hit_geometry {
                    self.invalidation.insert(InvalidationFlags::GEOMETRY);
                }
            }
            _ => self.projections.semantics_dirty = true,
        }
        match op {
            Op::CreateElement { id, tag, attrs } => {
                self.create_element(*id, *tag, attrs);
            }
            Op::CreateText { id, text } => {
                let id = *id;
                let decl = Declared {
                    text: Some(Arc::from(*text)),
                    ..Declared::default()
                };
                self.node_store.create_leaf(id, decl);
                self.recompute_solid(id);
            }
            Op::AppendChild { parent, child } => {
                let Some(child) = self.node_store.append(*parent, *child) else {
                    return;
                };
                // Nodes are styled when created, before they have a parent.
                self.recompute_subtree(child);
            }
            Op::InsertBefore {
                parent,
                child,
                ref_id,
            } => {
                let Some(child) = self.node_store.insert_before(*parent, *child, *ref_id) else {
                    return;
                };
                self.recompute_subtree(child);
            }
            Op::RemoveChild { parent, child } => {
                if self.node_store.remove_child(*parent, *child) {
                    self.invalidation.insert(InvalidationFlags::LAYOUT);
                }
            }
            Op::ReplaceNode {
                parent,
                old_id,
                new_id,
            } => {
                let Some(new) = self.node_store.replace(*parent, *old_id, *new_id) else {
                    return;
                };
                self.recompute_subtree(new);
            }
            Op::SetText { id, text } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    if let Some(d) = self.node_store.declared.get_mut(&n) {
                        d.text = Some(Arc::from(*text));
                    }
                    self.recompute_node(n);
                }
            }
            Op::SetClassName { id, classes } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    if let Some(d) = self.node_store.declared.get_mut(&n) {
                        d.classes.clone_from(classes);
                    }
                    self.recompute_node(n);
                }
            }
            Op::SetWidgetConfig { id, json } => {
                self.set_widget_config(*id, json);
            }
            Op::RemoveWidgetConfig { id } => {
                self.remove_widget_config(*id);
            }
            Op::SetStyle { id, prop, value } => {
                self.set_inline_ir(*id, *prop, style::parse_ir_value(value));
            }
            Op::SetStyleValue { id, prop, value } => {
                self.set_inline_ir(*id, *prop, style_value_ir(*value));
            }
            Op::SetShadows { id, shadows } => {
                self.set_shadows(*id, shadows);
            }
            Op::SetTransform2D { id, matrix } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    self.runtime_transforms.insert(n, *matrix);
                    if let Some(paint) = self.node_store.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.runtime_transform = Some(*matrix);
                        let _ = self.node_store.tree.set_node_context(n, Some(paint));
                    }
                }
            }
            Op::SetOverlayPlane { id, plane } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    let plane = match plane {
                        0 => OverlayPlane::Content,
                        1 => OverlayPlane::Floating,
                        2 => OverlayPlane::Modal,
                        // System and debug are intentionally host-reserved.
                        _ => OverlayPlane::Content,
                    };
                    self.overlay_planes.insert(n, plane);
                    if let Some(paint) = self.node_store.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.overlay_plane = plane;
                        let _ = self.node_store.tree.set_node_context(n, Some(paint));
                    }
                    self.invalidation.insert(InvalidationFlags::LAYOUT);
                }
            }
            Op::SetScrollbarStyle {
                id,
                visibility,
                hide_delay,
                fade_duration,
                thickness,
                margin,
                min_thumb_length,
                radius,
                colors,
            } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    let color = |rgba| {
                        Color::from_rgba8(
                            (rgba >> 24) as u8,
                            (rgba >> 16) as u8,
                            (rgba >> 8) as u8,
                            rgba as u8,
                        )
                    };
                    let style = ScrollbarStyle {
                        visibility: match visibility {
                            1 => ScrollbarVisibility::Always,
                            2 => ScrollbarVisibility::Hidden,
                            _ => ScrollbarVisibility::Auto,
                        },
                        hide_delay: Duration::from_secs_f32(*hide_delay / 1000.0),
                        fade_duration: Duration::from_secs_f32(*fade_duration / 1000.0),
                        thickness: *thickness,
                        margin: *margin,
                        min_thumb_length: *min_thumb_length,
                        radius: *radius,
                        track_color: color(colors[0]),
                        thumb_color: color(colors[1]),
                        hover_color: color(colors[2]),
                        active_color: color(colors[3]),
                    };
                    self.scroll.styles.insert(n, style);
                    if let Some(paint) = self.node_store.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.scrollbar = style;
                        let _ = self.node_store.tree.set_node_context(n, Some(paint));
                    }
                }
            }
            Op::FocusNode { id } => {
                if self.node_store.solid_to_node.contains_key(id) {
                    self.set_focused_target(Some(*id));
                }
            }
            Op::ScrollTo { id, x, y } => {
                self.scroll_node(*id, *x, *y, false);
            }
            Op::ScrollBy { id, x, y } => {
                self.scroll_node(*id, *x, *y, true);
            }
            Op::RemoveStyle { id, prop } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    if let Some(d) = self.node_store.declared.get_mut(&n) {
                        d.inline.remove(prop);
                    }
                    self.recompute_node(n);
                }
            }
            Op::SetAttribute { id, name, value } => {
                self.set_attribute(*id, *name, value);
            }
            Op::RemoveAttribute { id, name } => {
                self.remove_attribute(*id, *name);
            }
            Op::AddEventListener { id, event_type } => {
                self.input
                    .listeners
                    .entry(*id)
                    .or_default()
                    .insert(*event_type);
            }
            Op::RemoveEventListener { id, event_type } => {
                if let Some(s) = self.input.listeners.get_mut(id) {
                    s.remove(*event_type);
                }
            }
            Op::DropNode { id } => {
                self.drop_node(*id);
            }
            Op::CreateComment { .. } | Op::FrameEnd => {}
        }
    }
}
