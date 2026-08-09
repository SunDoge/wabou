use super::*;

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
        self.projections.semantics_dirty = true;
        match op {
            Op::CreateElement { id, tag, attrs } => {
                let id = *id;
                let mut decl = Declared {
                    tag: Some(*tag),
                    ..Declared::default()
                };
                let class_value = {
                    let atoms = self.atoms.borrow();
                    if atoms.resolve(*tag).is_none() {
                        tracing::warn!(atom = tag.get(), "unknown tag atom");
                    }
                    attrs.iter().find_map(|(name, value)| {
                        matches!(atoms.resolve(*name), Some("class" | "className"))
                            .then_some(*value)
                    })
                };
                if let Some(value) = class_value {
                    // CreateElement attributes are retained for protocol
                    // compatibility; class tokens normally arrive through
                    // SetClassName and are already atoms there.
                    let mut atoms = self.atoms.borrow_mut();
                    decl.classes = value
                        .split_whitespace()
                        .map(|value| atoms.intern(value))
                        .collect();
                }
                for (name, value) in attrs {
                    decl.attrs.insert(*name, Arc::from(*value));
                }
                let node = self.node_store.create_leaf(id, decl);
                self.recompute_solid(id);
                // Rust-side widget creation: when the tag matches a known widget
                // type, create + store it. The widget paints custom content
                // (shapes, text+caret, …) that the standard renderer can't.
                if let Some(mut widget) = self
                    .widget_manager
                    .create(*tag, self.wake_callback.as_ref())
                {
                    // Feed initial attrs to the widget so it receives JS params.
                    let atoms = self.atoms.borrow();
                    for (name, value) in attrs {
                        if let Some(n) = atoms.resolve(*name) {
                            widget.attribute_changed(n, value);
                        }
                    }
                    drop(atoms);
                    self.widget_manager.widgets.insert(node, widget);
                    self.recompute_node(node);
                }
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
            Op::SetStyle { id, prop, value } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    let Some(property) = self.inline_property(*prop) else {
                        tracing::warn!(atom = prop.get(), "unknown style-property atom");
                        return;
                    };
                    let ir = style::parse_ir_value(value);
                    if let Some(d) = self.node_store.declared.get_mut(&n) {
                        d.inline.insert(*prop, InlineValue::Typed(ir.clone()));
                    }
                    // Fast path: a non-inherited inline property can be applied
                    // directly to the existing (post-inherit) ComputedStyle —
                    // the class rules haven't changed, so re-resolving them is
                    // wasted work, and skipping it also lets the layout branch
                    // skip the O(N) inherit pass. Inherited properties (color,
                    // font-*) still need the slow path to propagate to
                    // descendants. This is the hot path for per-frame animation
                    // (e.g. moving N nodes via top/left = 2N SetStyles/frame).
                    if property.inherited {
                        self.recompute_node(n);
                    } else if !self.apply_inline_ir_fast(n, &property.name, &ir) {
                        if let Some(d) = self.node_store.declared.get_mut(&n) {
                            d.inline.remove(prop);
                        }
                        if self.warned_ir_properties.insert(*prop) {
                            tracing::warn!(property = %property.name, "unsupported inline style property or value");
                        }
                    }
                }
            }
            Op::SetStyleValue { id, prop, value } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    let Some(property) = self.inline_property(*prop) else {
                        tracing::warn!(atom = prop.get(), "unknown style-property atom");
                        return;
                    };
                    let ir = match value {
                        crate::protocol::StyleValue::Px(value) => IrValue::Length {
                            value: wabou_shell::style::IrLength::Px { value: *value },
                        },
                        crate::protocol::StyleValue::Percent(value) => IrValue::Length {
                            value: wabou_shell::style::IrLength::Percent { value: *value },
                        },
                        crate::protocol::StyleValue::Number(value) => {
                            IrValue::Number { value: *value }
                        }
                        crate::protocol::StyleValue::Boolean(value) => {
                            IrValue::Boolean { value: *value }
                        }
                        crate::protocol::StyleValue::Color(rgba) => IrValue::Color {
                            value: wabou_shell::style::IrColor::Literal { rgba: *rgba },
                        },
                        crate::protocol::StyleValue::Auto => IrValue::Length {
                            value: wabou_shell::style::IrLength::Auto,
                        },
                    };
                    if let Some(d) = self.node_store.declared.get_mut(&n) {
                        d.inline.insert(*prop, InlineValue::Typed(ir.clone()));
                    }
                    if property.inherited {
                        self.recompute_node(n);
                    } else if !self.apply_inline_ir_fast(n, &property.name, &ir) {
                        if let Some(d) = self.node_store.declared.get_mut(&n) {
                            d.inline.remove(prop);
                        }
                        if self.warned_ir_properties.insert(*prop) {
                            tracing::warn!(property = %property.name, "unsupported typed inline style property or value");
                        }
                    }
                }
            }
            Op::SetShadows { id, shadows } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
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
                                        value: wabou_shell::style::IrColor::Literal {
                                            rgba: shadow.color,
                                        },
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
                    if let Some(declared) = self.node_store.declared.get_mut(&n) {
                        declared.inline.insert(prop, InlineValue::Typed(ir.clone()));
                    }
                    if !self.apply_inline_ir_fast(n, "box-shadow", &ir) {
                        if let Some(declared) = self.node_store.declared.get_mut(&n) {
                            declared.inline.remove(&prop);
                        }
                        tracing::warn!("invalid Vello shadow list");
                    }
                }
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
                        thickness: *thickness,
                        margin: *margin,
                        min_thumb_length: *min_thumb_length,
                        radius: *radius,
                        track_color: color(colors[0]),
                        thumb_color: color(colors[1]),
                        hover_color: color(colors[2]),
                        active_color: color(colors[3]),
                    };
                    self.scrollbar_styles.insert(n, style);
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
            Op::SetAttribute { id, name, value }
                if matches!(
                    self.atoms.borrow().resolve(*name),
                    Some("class" | "className")
                ) =>
            {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    if let Some(d) = self.node_store.declared.get_mut(&n) {
                        let mut atoms = self.atoms.borrow_mut();
                        d.classes = value
                            .split_whitespace()
                            .map(|value| atoms.intern(value))
                            .collect();
                        d.attrs.insert(*name, Arc::from(*value));
                    }
                    self.recompute_node(n);
                }
            }
            Op::SetAttribute { id, name, value } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    if let Some(d) = self.node_store.declared.get_mut(&n) {
                        d.attrs.insert(*name, Arc::from(*value));
                    }
                    // Forward attribute changes to Rust-side widgets.
                    if let Some(widget) = self.widget_manager.widgets.get_mut(&n) {
                        let atoms = self.atoms.borrow();
                        if let Some(n_str) = atoms.resolve(*name) {
                            widget.attribute_changed(n_str, value);
                        }
                    }
                    self.recompute_node(n);
                }
            }
            Op::RemoveAttribute { id, name } => {
                if let Some(&n) = self.node_store.solid_to_node.get(id) {
                    let is_class = matches!(
                        self.atoms.borrow().resolve(*name),
                        Some("class" | "className")
                    );
                    if let Some(d) = self.node_store.declared.get_mut(&n) {
                        d.attrs.remove(name);
                        if is_class {
                            d.classes.clear();
                        }
                    }
                    if let Some(widget) = self.widget_manager.widgets.get_mut(&n)
                        && let Some(n_str) = self.atoms.borrow().resolve(*name)
                    {
                        widget.attribute_removed(n_str);
                    }
                    self.recompute_node(n);
                }
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
                if self.input.pointer_down_target == Some(*id) {
                    self.cancel_active_pointer_gesture();
                }
                let node = self.node_store.solid_to_node.get(id).copied();
                let selection_dropped = self.active_text_selection.as_ref().is_some_and(|active| {
                    active.anchor_target == *id || active.focus_target == *id
                });
                if selection_dropped {
                    self.active_text_selection = None;
                    self.next_text_selection_scroll = None;
                    self.sync_text_selection_change();
                }
                if self.input.focused_target == Some(*id) {
                    if self.input.window_focused
                        && let Some(widget) =
                            node.and_then(|node| self.widget_manager.widgets.get_mut(&node))
                    {
                        widget.focus_changed(false);
                    }
                    self.input.focused_target = None;
                }
                self.input.listeners.remove(id);
                self.resize_targets.borrow_mut().remove(id);
                // Keep the cached hover/focus targets from dangling at a solid
                // id whose node was just torn down — a stale hit would make
                // wheel/scroll (and keyboard delivery) silently no-op until a
                // pointer move re-establishes the hit.
                if self.input.hovered_target == Some(*id) {
                    self.input.hovered_target = None;
                }
                if let Some(n) = self.node_store.remove(*id) {
                    self.runtime_transforms.remove(&n);
                    self.overlay_planes.remove(&n);
                    self.scrollbar_styles.remove(&n);
                    self.scroll_offsets.remove(&n);
                    self.svg_cache.remove(&n);
                    self.style_diagnostics.remove(&n);
                    if let Some(widget) = self.widget_manager.widgets.get_mut(&n) {
                        widget.unmount();
                    }
                    self.drain_widget_host_actions(n);
                    self.widget_manager.widgets.remove(&n);
                    self.widget_manager.styles.remove(&n);
                    self.widget_manager
                        .host_action_routes
                        .retain(|_, (widget_node, _)| *widget_node != n);
                    self.invalidation.insert(InvalidationFlags::LAYOUT);
                }
            }
            Op::CreateComment { .. } | Op::FrameEnd => {}
        }
    }
}
