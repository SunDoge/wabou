use super::*;

fn decode_vector_path(data: &[u8]) -> Option<Arc<wabou_shell::style::VectorPath>> {
    use vello::kurbo::{BezPath, Cap, Join, Stroke};

    const MAGIC: u32 = 0x3150_4257;
    if data.len() < 36
        || u32::from_le_bytes(data[0..4].try_into().ok()?) != MAGIC
        || u16::from_le_bytes(data[4..6].try_into().ok()?) != 1
        || u16::from_le_bytes(data[6..8].try_into().ok()?) != 0
        || u32::from_le_bytes(data[12..16].try_into().ok()?) as usize != data.len()
    {
        return None;
    }
    let count = u32::from_le_bytes(data[8..12].try_into().ok()?) as usize;
    if count > (data.len() - 36) / 4 {
        return None;
    }
    let fill = u32::from_le_bytes(data[16..20].try_into().ok()?);
    let stroke = u32::from_le_bytes(data[20..24].try_into().ok()?);
    let stroke_width = f32::from_le_bytes(data[24..28].try_into().ok()?);
    let fill_rule = data[28];
    let line_cap = data[29];
    let line_join = data[30];
    let miter_limit = f32::from_le_bytes(data[32..36].try_into().ok()?);
    if data[31] != 0
        || !stroke_width.is_finite()
        || stroke_width <= 0.0
        || !miter_limit.is_finite()
        || miter_limit <= 0.0
        || fill_rule > 1
        || line_cap > 2
        || line_join > 2
    {
        return None;
    }
    let mut offset = 36usize;
    let mut path = BezPath::new();
    let mut has_subpath = false;
    let read = |offset: &mut usize| -> Option<f64> {
        let end = offset.checked_add(4)?;
        let value = f32::from_le_bytes(data.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        value.is_finite().then_some(f64::from(value))
    };
    for _ in 0..count {
        let command = *data.get(offset)?;
        if data.get(offset + 1..offset + 4)? != [0, 0, 0] {
            return None;
        }
        offset += 4;
        match command {
            1 => {
                path.move_to((read(&mut offset)?, read(&mut offset)?));
                has_subpath = true;
            }
            2 if has_subpath => path.line_to((read(&mut offset)?, read(&mut offset)?)),
            3 if has_subpath => path.quad_to(
                (read(&mut offset)?, read(&mut offset)?),
                (read(&mut offset)?, read(&mut offset)?),
            ),
            4 if has_subpath => path.curve_to(
                (read(&mut offset)?, read(&mut offset)?),
                (read(&mut offset)?, read(&mut offset)?),
                (read(&mut offset)?, read(&mut offset)?),
            ),
            5 if has_subpath => {
                path.close_path();
                has_subpath = false;
            }
            _ => return None,
        }
    }
    if offset != data.len() || path.is_empty() {
        return None;
    }
    let color = |rgba: u32| {
        ((rgba & 0xff) != 0).then(|| {
            Color::from_rgba8(
                (rgba >> 24) as u8,
                (rgba >> 16) as u8,
                (rgba >> 8) as u8,
                rgba as u8,
            )
        })
    };
    let cap = match line_cap {
        1 => Cap::Round,
        2 => Cap::Square,
        _ => Cap::Butt,
    };
    let join = match line_join {
        1 => Join::Round,
        2 => Join::Bevel,
        _ => Join::Miter,
    };
    Some(Arc::new(wabou_shell::style::VectorPath {
        path: Arc::new(path),
        fill: color(fill),
        stroke: color(stroke),
        even_odd: fill_rule == 1,
        stroke_style: Stroke::new(f64::from(stroke_width))
            .with_caps(cap)
            .with_join(join)
            .with_miter_limit(f64::from(miter_limit)),
    }))
}

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
    fn project_structure_if_unbatched(&mut self) {
        if !self.document.applying_frame && self.document.ifc_dirty {
            self.rebuild_layout_boxes();
        }
    }

    fn inline_property(&mut self, atom: Atom) -> Option<InlineProperty> {
        if let Some(property) = self.document.style.inline_properties.get(&atom) {
            return Some(property.clone());
        }
        let name: Arc<str> = {
            let atoms = self.document.atoms.borrow();
            Arc::from(atoms.resolve(atom)?)
        };
        let inherited = INHERITED_PROPERTIES.contains(&name.as_ref()) || name.as_ref() == "font";
        let property = InlineProperty { name, inherited };
        self.document
            .style
            .inline_properties
            .insert(atom, property.clone());
        Some(property)
    }

    fn is_in_svg_subtree(&self, mut node: NodeId) -> bool {
        let atoms = self.document.atoms.borrow();
        loop {
            if self
                .document
                .node_store
                .declared
                .get(&node)
                .and_then(|declared| declared.tag)
                .and_then(|tag| atoms.resolve(tag))
                == Some("svg")
            {
                return true;
            }
            let Some(parent) = self.document.node_store.logical_parent.get(&node).copied() else {
                return false;
            };
            node = parent;
        }
    }

    fn set_inline_ir(&mut self, id: NodeKey, prop: Atom, ir: IrValue) {
        let Some(&node) = self.document.node_store.solid_to_node.get(&id) else {
            return;
        };
        let Some(property) = self.inline_property(prop) else {
            tracing::warn!(atom = prop.get(), "unknown style-property atom");
            return;
        };
        if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
            declared.inline.insert(prop, InlineValue::Typed(ir.clone()));
        }

        // Non-inherited values can update the post-inheritance computed style
        // directly. Inherited values must propagate through the subtree.
        if property.inherited {
            self.recompute_node(node);
        } else if !self.apply_inline_ir_fast(node, &property.name, &ir) {
            if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                declared.inline.remove(&prop);
            }
            if self.document.style.warned_ir_properties.insert(prop) {
                tracing::warn!(property = %property.name, "unsupported inline style property or value");
            }
        }
    }

    fn create_element(&mut self, id: NodeKey, tag: Atom) {
        let declared = Declared {
            tag: Some(tag),
            ..Declared::default()
        };
        if self.document.atoms.borrow().resolve(tag).is_none() {
            tracing::warn!(atom = tag.get(), "unknown tag atom");
        }

        let node = self.document.node_store.create_leaf(id, declared);
        self.recompute_solid(id);
        let Some(mut widget) = self
            .document
            .widget_manager
            .create(tag, self.runtime.wake_callback.as_ref())
        else {
            return;
        };
        let mounted_changes = widget.mounted();
        self.document.widget_manager.widgets.insert(node, widget);
        self.document.ifc_dirty = true;
        self.invalidate_widget_changes(mounted_changes);
        self.drain_widget_host_actions(node);
        self.drain_widget_node_events(node);
        self.recompute_node(node);
    }

    fn set_shadows(&mut self, id: NodeKey, shadows: &[crate::protocol::ShadowValue]) {
        let Some(&node) = self.document.node_store.solid_to_node.get(&id) else {
            return;
        };
        let prop = self.document.atoms.borrow_mut().intern("box-shadow");
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
        if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
            declared.inline.insert(prop, InlineValue::Typed(ir.clone()));
        }
        if !self.apply_inline_ir_fast(node, "box-shadow", &ir) {
            if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                declared.inline.remove(&prop);
            }
            tracing::warn!("invalid Vello shadow list");
        }
    }

    fn drop_node(&mut self, id: NodeKey) {
        if self.interaction.input.pointer_down_target == Some(id) {
            self.cancel_active_pointer_gesture();
        }
        let node = self.document.node_store.solid_to_node.get(&id).copied();
        if self
            .interaction
            .text_selection
            .active
            .as_ref()
            .is_some_and(|active| active.anchor_target == id || active.focus_target == id)
        {
            self.interaction.text_selection.active = None;
            self.interaction.text_selection.next_scroll = None;
            self.sync_text_selection_change();
        }
        if self.interaction.input.focused_target == Some(id) {
            if self.interaction.input.window_focused
                && let Some(widget) =
                    node.and_then(|node| self.document.widget_manager.widgets.get_mut(&node))
            {
                widget.focus_changed(false);
            }
            self.interaction.input.focused_target = None;
        }
        self.interaction.input.listeners.remove(&id);
        self.interaction.scroll.pending_events.remove(&id);
        self.frame.resize_targets.borrow_mut().remove(&id);
        // Never retain an id after its generational node has been removed.
        if self.interaction.input.hovered_target == Some(id) {
            self.interaction.input.hovered_target = None;
        }

        let Some(node) = self.document.node_store.remove(id) else {
            return;
        };
        self.document.runtime_transforms.remove(&node);
        self.document.overlay_planes.remove(&node);
        self.interaction.scroll.styles.remove(&node);
        self.interaction.scroll.offsets.remove(&node);
        self.document.resources.svg.remove(&node);
        self.document.style.diagnostics.remove(&node);
        #[cfg(any(feature = "devtools", test))]
        self.document.style.cascade.remove(&node);
        if let Some(widget) = self.document.widget_manager.widgets.get_mut(&node) {
            widget.unmount();
        }
        self.drain_widget_host_actions(node);
        self.document.widget_manager.widgets.remove(&node);
        self.document.widget_manager.styles.remove(&node);
        self.document.widget_manager.geometries.remove(&node);
        self.document.widget_manager.visibility.remove(&node);
        self.document
            .widget_manager
            .host_action_routes
            .retain(|_, (widget_node, _)| *widget_node != node);
        self.document.invalidation.insert(InvalidationFlags::LAYOUT);
        self.document.ifc_dirty = true;
    }

    fn set_attribute(&mut self, id: NodeKey, name: Atom, value: &str) {
        let Some(&node) = self.document.node_store.solid_to_node.get(&id) else {
            return;
        };
        let is_class = matches!(
            self.document.atoms.borrow().resolve(name),
            Some("class" | "className")
        );
        let affects_resolved_style = is_class || self.is_in_svg_subtree(node);
        if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
            if is_class {
                let mut atoms = self.document.atoms.borrow_mut();
                declared.classes = value
                    .split_whitespace()
                    .map(|value| atoms.intern(value))
                    .collect();
            }
            declared.attrs.insert(name, Arc::from(value));
        }
        let widget_changes = if !is_class
            && let Some(widget) = self.document.widget_manager.widgets.get_mut(&node)
            && let Some(name) = self.document.atoms.borrow().resolve(name)
        {
            widget.attribute_changed(name, value)
        } else {
            wabou_shell::WidgetChanges::empty()
        };
        self.invalidate_widget_changes(widget_changes);
        self.frame.projections.semantics_dirty = true;
        if affects_resolved_style {
            if self.is_in_svg_subtree(node) {
                self.document
                    .invalidation
                    .insert(InvalidationFlags::INHERIT);
            }
            self.recompute_node(node);
        }
    }

    fn set_widget_config(&mut self, id: NodeKey, json: &str) {
        let Some(&node) = self.document.node_store.solid_to_node.get(&id) else {
            return;
        };
        if let Some(widget) = self.document.widget_manager.widgets.get_mut(&node) {
            match widget.config_changed(json) {
                Ok(changes) => self.invalidate_widget_changes(changes),
                Err(error) => {
                    tracing::warn!(?id, %error, "widget rejected widgetConfig");
                }
            }
        }
    }

    fn remove_widget_config(&mut self, id: NodeKey) {
        let Some(&node) = self.document.node_store.solid_to_node.get(&id) else {
            return;
        };
        if let Some(widget) = self.document.widget_manager.widgets.get_mut(&node) {
            let changes = widget.config_removed();
            self.invalidate_widget_changes(changes);
        }
    }

    fn set_graphic_source(&mut self, id: NodeKey, kind: u8, source: &str) {
        let Some(&node) = self.document.node_store.solid_to_node.get(&id) else {
            return;
        };
        match kind {
            crate::protocol::GRAPHIC_SOURCE_SVG => {
                if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                    declared.svg_source = Some(Arc::from(source));
                }
            }
            crate::protocol::GRAPHIC_SOURCE_RESOURCE_RASTER => {
                let handle = source.split_once(':').and_then(|(lo, hi)| {
                    Some(crate::ImageResourceHandle {
                        lo: lo.parse().ok()?,
                        hi: hi.parse().ok()?,
                    })
                });
                let resource =
                    handle.and_then(|handle| self.document.resources.image_store.get(handle));
                if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                    declared.image_resource = resource.as_ref().and(handle);
                }
                if let Some(resource) = resource {
                    let (width, height) = resource.dimensions();
                    if let Some(&target) = self.document.node_store.node_to_solid.get(&node) {
                        self.dispatch_image_resource_ready(
                            target,
                            handle.expect("resolved resource has a handle"),
                            width as f32,
                            height as f32,
                        );
                    }
                } else {
                    tracing::warn!(source, "rejected missing or stale image resource handle");
                    self.dispatch_image_resource_error(
                        node,
                        handle,
                        "image resource is missing or stale",
                    );
                }
            }
            _ => unreachable!("graphic source kind was validated by the decoder"),
        }
        self.recompute_node(node);
    }

    fn clear_graphic_source(&mut self, id: NodeKey, kind: u8) {
        let Some(&node) = self.document.node_store.solid_to_node.get(&id) else {
            return;
        };
        match kind {
            crate::protocol::GRAPHIC_SOURCE_SVG => {
                if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                    declared.svg_source = None;
                }
            }
            crate::protocol::GRAPHIC_SOURCE_RESOURCE_RASTER => {
                if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                    declared.image_resource = None;
                }
            }
            _ => unreachable!("graphic source kind was validated by the decoder"),
        }
        self.recompute_node(node);
    }

    fn remove_attribute(&mut self, id: NodeKey, name: Atom) {
        let Some(&node) = self.document.node_store.solid_to_node.get(&id) else {
            return;
        };
        let is_class = matches!(
            self.document.atoms.borrow().resolve(name),
            Some("class" | "className")
        );
        let affects_resolved_style = is_class || self.is_in_svg_subtree(node);
        if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
            declared.attrs.remove(&name);
            if is_class {
                declared.classes.clear();
            }
        }
        let widget_changes = if let Some(widget) =
            self.document.widget_manager.widgets.get_mut(&node)
            && let Some(name) = self.document.atoms.borrow().resolve(name)
        {
            widget.attribute_removed(name)
        } else {
            wabou_shell::WidgetChanges::empty()
        };
        self.invalidate_widget_changes(widget_changes);
        self.frame.projections.semantics_dirty = true;
        if affects_resolved_style {
            if self.is_in_svg_subtree(node) {
                self.document
                    .invalidation
                    .insert(InvalidationFlags::INHERIT);
            }
            self.recompute_node(node);
        }
    }

    /// Decode + apply one frame's ops in order.
    pub(super) fn apply_frame(&mut self, frame: &Frame) {
        self.document.applying_frame = true;
        for op in &frame.ops {
            self.apply_op(op);
        }
        self.document.applying_frame = false;
        let dirty = std::mem::take(&mut self.document.dirty_styles);
        for node in dirty {
            self.recompute_node_now(node);
        }
        if self.document.ifc_dirty {
            self.rebuild_layout_boxes();
        }
    }

    pub(super) fn apply_op(&mut self, op: &Op) {
        #[cfg(any(feature = "devtools", test))]
        {
            self.frame.projections.debug_dirty |= self.frame.projections.debug_state.is_some();
        }
        match op {
            Op::CreateElement { id, tag } => {
                self.create_element(*id, *tag);
            }
            Op::CreateText { id, text } => {
                let id = *id;
                let decl = Declared {
                    text: Some(Arc::from(*text)),
                    ..Declared::default()
                };
                self.document.node_store.create_leaf(id, decl);
                self.recompute_solid(id);
            }
            Op::AppendChild { parent, child } => {
                let Some(child) = self.document.node_store.append(*parent, *child) else {
                    return;
                };
                // Nodes are styled when created, before they have a parent.
                self.recompute_subtree(child);
                self.document.ifc_dirty = true;
                self.frame.projections.semantics_dirty = true;
                self.document
                    .invalidation
                    .insert(InvalidationFlags::INHERIT);
            }
            Op::InsertBefore {
                parent,
                child,
                ref_id,
            } => {
                let Some(child) = self
                    .document
                    .node_store
                    .insert_before(*parent, *child, *ref_id)
                else {
                    return;
                };
                self.recompute_subtree(child);
                self.document.ifc_dirty = true;
                self.frame.projections.semantics_dirty = true;
                self.document
                    .invalidation
                    .insert(InvalidationFlags::INHERIT);
            }
            Op::RemoveChild { parent, child } => {
                if self.document.node_store.remove_child(*parent, *child) {
                    self.document.invalidation.insert(InvalidationFlags::LAYOUT);
                    self.document.ifc_dirty = true;
                    self.frame.projections.semantics_dirty = true;
                }
            }
            Op::SetText { id, text } => {
                if let Some(&n) = self.document.node_store.solid_to_node.get(id) {
                    if let Some(d) = self.document.node_store.declared.get_mut(&n) {
                        d.text = Some(Arc::from(*text));
                    }
                    self.document.ifc_dirty = true;
                    self.frame.projections.semantics_dirty = true;
                    self.recompute_node(n);
                }
            }
            Op::SetClassName { id, classes } => {
                if let Some(&n) = self.document.node_store.solid_to_node.get(id) {
                    if let Some(d) = self.document.node_store.declared.get_mut(&n) {
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
            Op::SetTextBehavior { id, flags } => {
                if let Some(&node) = self.document.node_store.solid_to_node.get(id) {
                    if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                        declared.text_behavior = *flags;
                    }
                    self.document.ifc_dirty = true;
                    self.recompute_node(node);
                }
            }
            Op::SetTextMaxLines { id, max_lines } => {
                if let Some(&node) = self.document.node_store.solid_to_node.get(id) {
                    if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                        declared.text_max_lines = *max_lines;
                    }
                    self.recompute_node(node);
                }
            }
            Op::SetInteractionPolicy {
                id,
                flags,
                focus_order,
            } => {
                if let Some(&node) = self.document.node_store.solid_to_node.get(id)
                    && let Some(declared) = self.document.node_store.declared.get_mut(&node)
                {
                    declared.focus_order = (flags & crate::protocol::INTERACTION_POLICY_FOCUSABLE
                        != 0)
                        .then_some(*focus_order);
                    declared.interaction_blocked =
                        flags & crate::protocol::INTERACTION_POLICY_BLOCK_SUBTREE != 0;
                    declared.focus_contained =
                        flags & crate::protocol::INTERACTION_POLICY_CONTAIN_FOCUS != 0;
                    self.frame.projections.semantics_dirty = true;
                }
            }
            Op::SetGraphicSource { id, kind, source } => {
                self.set_graphic_source(*id, *kind, source);
            }
            Op::ClearGraphicSource { id, kind } => {
                self.clear_graphic_source(*id, *kind);
            }
            Op::SetGraphicData { id, kind, data } => {
                if let Some(&node) = self.document.node_store.solid_to_node.get(id) {
                    debug_assert_eq!(*kind, crate::protocol::GRAPHIC_DATA_VECTOR_PATH);
                    let decoded = decode_vector_path(data);
                    if decoded.is_none() {
                        tracing::warn!(node = %id, "rejected malformed vector path");
                    }
                    if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                        declared.vector_path = decoded;
                    }
                    self.recompute_node(node);
                }
            }
            Op::ClearGraphicData { id, kind } => {
                debug_assert_eq!(*kind, crate::protocol::GRAPHIC_DATA_VECTOR_PATH);
                if let Some(&node) = self.document.node_store.solid_to_node.get(id) {
                    if let Some(declared) = self.document.node_store.declared.get_mut(&node) {
                        declared.vector_path = None;
                    }
                    self.recompute_node(node);
                }
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
                if let Some(&n) = self.document.node_store.solid_to_node.get(id) {
                    let affects_hit_geometry = self
                        .document
                        .node_store
                        .tree
                        .get_node_context(n)
                        .is_some_and(|paint| paint.pointer_events)
                        || self
                            .document
                            .node_store
                            .children
                            .get(&n)
                            .is_some_and(|children| !children.is_empty());
                    self.document.runtime_transforms.insert(n, *matrix);
                    if let Some(paint) = self.document.node_store.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.runtime_transform = Some(*matrix);
                        let _ = self
                            .document
                            .node_store
                            .tree
                            .set_node_context(n, Some(paint));
                    }
                    if affects_hit_geometry {
                        self.document
                            .invalidation
                            .insert(InvalidationFlags::GEOMETRY);
                    }
                }
            }
            Op::SetOverlayPlane { id, plane } => {
                if let Some(&n) = self.document.node_store.solid_to_node.get(id) {
                    let plane = match plane {
                        0 => OverlayPlane::Content,
                        1 => OverlayPlane::Floating,
                        2 => OverlayPlane::Modal,
                        // System and debug are intentionally host-reserved.
                        _ => OverlayPlane::Content,
                    };
                    self.document.overlay_planes.insert(n, plane);
                    if let Some(paint) = self.document.node_store.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.overlay_plane = plane;
                        let _ = self
                            .document
                            .node_store
                            .tree
                            .set_node_context(n, Some(paint));
                    }
                    self.document.invalidation.insert(InvalidationFlags::LAYOUT);
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
                if let Some(&n) = self.document.node_store.solid_to_node.get(id) {
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
                    self.interaction.scroll.styles.insert(n, style);
                    if let Some(paint) = self.document.node_store.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.scrollbar = style;
                        let _ = self
                            .document
                            .node_store
                            .tree
                            .set_node_context(n, Some(paint));
                    }
                    self.document
                        .invalidation
                        .insert(InvalidationFlags::GEOMETRY);
                }
            }
            Op::FocusNode { id } => {
                if self.document.node_store.solid_to_node.contains_key(id) {
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
                if let Some(&n) = self.document.node_store.solid_to_node.get(id) {
                    if let Some(d) = self.document.node_store.declared.get_mut(&n) {
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
                self.interaction
                    .input
                    .listeners
                    .entry(*id)
                    .or_default()
                    .insert(*event_type);
            }
            Op::RemoveEventListener { id, event_type } => {
                if let Some(s) = self.interaction.input.listeners.get_mut(id) {
                    s.remove(*event_type);
                }
            }
            Op::DropNode { id } => {
                self.frame.projections.semantics_dirty = true;
                self.drop_node(*id);
            }
        }
        self.project_structure_if_unbatched();
    }
}

#[cfg(test)]
mod vector_path_tests;
