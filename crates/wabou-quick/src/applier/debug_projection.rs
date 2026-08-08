use super::*;

impl Applier {
    pub(super) fn publish_layout_metrics(&self, placed: &[PlacedNode], width: u32, height: u32) {
        let viewport = LayoutRect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };
        let mut snapshot = self.projections.layout_metrics.borrow_mut();
        snapshot.revision = snapshot.revision.wrapping_add(1);
        snapshot.viewport = viewport;
        snapshot.nodes.clear();
        snapshot.nodes.reserve(placed.len());
        for placed_node in placed {
            let Some(&id) = self.node_store.node_to_solid.get(&placed_node.node_id) else {
                continue;
            };
            let rect = |value: [f32; 4]| LayoutRect {
                x: value[0],
                y: value[1],
                width: (value[2] - value[0]).max(0.0),
                height: (value[3] - value[1]).max(0.0),
            };
            snapshot.nodes.insert(
                id,
                LayoutMetric {
                    rect: rect(placed_node.rect),
                    clip: placed_node.clip.map_or(viewport, rect),
                },
            );
        }
    }

    pub(super) fn publish_debug_snapshot(&mut self, placed: &[PlacedNode]) {
        let Some(state) = self.projections.debug_state.clone() else {
            return;
        };
        self.projections.debug_revision = self.projections.debug_revision.wrapping_add(1);
        let atoms = self.atoms.borrow();
        let placed_by_id: HashMap<_, _> = placed.iter().map(|node| (node.node_id, node)).collect();
        let mut css_transforms = HashMap::with_capacity(placed.len());
        for node in placed {
            let parent_transform = node
                .parent_node_id
                .and_then(|parent| css_transforms.get(&parent).copied())
                .unwrap_or(Affine::IDENTITY);
            css_transforms.insert(
                node.node_id,
                wabou_shell::scene::resolve_node_transform(node, parent_transform),
            );
        }
        let mut nodes = Vec::with_capacity(placed.len());
        for placed_node in placed {
            let Some(&id) = self.node_store.node_to_solid.get(&placed_node.node_id) else {
                continue;
            };
            let declared = self.node_store.declared.get(&placed_node.node_id);
            let tag = declared
                .and_then(|declared| declared.tag)
                .and_then(|tag| atoms.resolve(tag))
                .unwrap_or(if id == 1 { "#root" } else { "#text" })
                .to_owned();
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
            let mut listeners: Vec<_> = self
                .input
                .listeners
                .get(&id)
                .into_iter()
                .flat_map(|events| events.codes())
                .collect();
            listeners.sort_unstable();
            let classes = declared
                .into_iter()
                .flat_map(|declared| declared.classes.iter())
                .filter_map(|class| atoms.resolve(*class).map(str::to_owned))
                .collect();
            let matched_rules = declared
                .into_iter()
                .flat_map(|declared| {
                    std::iter::once(&self.universal_rules).chain(
                        declared
                            .classes
                            .iter()
                            .filter_map(|class| self.rule_index.get(class)),
                    )
                })
                .flatten()
                .filter_map(|index| self.style_ir.as_ref()?.rules.get(*index))
                .map(|rule| {
                    if rule.class_name == "*" {
                        "*".to_owned()
                    } else {
                        format!(".{}", rule.class_name)
                    }
                })
                .collect();
            let [x0, y0, x1, y1] = placed_node.rect;
            let [cx, cy] = placed_node.content_origin;
            let [cw, ch] = placed_node.content_size;
            let content_transform =
                css_transforms[&placed_node.node_id] * Affine::translate((cx as f64, cy as f64));
            let (static_transform, _) = wabou_shell::scene::resolve_local_transforms(placed_node);
            let layout = self.node_store.tree.style(placed_node.node_id).ok();
            let debug_rect = |[x0, y0, x1, y1]: [f32; 4]| wabou_devtools::Rect {
                x: x0,
                y: y0,
                width: (x1 - x0).max(0.0),
                height: (y1 - y0).max(0.0),
            };
            let intersect = |left: [f32; 4], right: [f32; 4]| {
                [
                    left[0].max(right[0]),
                    left[1].max(right[1]),
                    left[2].min(right[2]),
                    left[3].min(right[3]),
                ]
            };
            let mut clip_ancestors = Vec::new();
            let mut ancestor_id = placed_node.parent_node_id;
            while let Some(node_id) = ancestor_id {
                let Some(ancestor) = placed_by_id.get(&node_id).copied() else {
                    break;
                };
                if let Some(rect) = ancestor.own_clip {
                    clip_ancestors.push(wabou_devtools::DebugClip {
                        node_id: self
                            .node_store
                            .node_to_solid
                            .get(&node_id)
                            .copied()
                            .unwrap_or(0),
                        kind: "ancestor-overflow".into(),
                        coordinate_space: "layout-window-logical".into(),
                        rect: debug_rect(rect),
                        radius: ancestor.own_clip_radius,
                        transform: css_transforms[&node_id].as_coeffs(),
                    });
                }
                ancestor_id = ancestor.parent_node_id;
            }
            clip_ancestors.reverse();
            if let Some(rect) = placed_node.own_clip {
                clip_ancestors.push(wabou_devtools::DebugClip {
                    node_id: id,
                    kind: "self-overflow".into(),
                    coordinate_space: "layout-window-logical".into(),
                    rect: debug_rect(rect),
                    radius: placed_node.own_clip_radius,
                    transform: css_transforms[&placed_node.node_id].as_coeffs(),
                });
            }
            let widget_local = self
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
                (Some(inherited), Some((local, _))) => Some((intersect(inherited, local), 0.0)),
                (Some(inherited), None) => Some((inherited, placed_node.clip_radius)),
                (None, Some(local)) => Some(local),
                (None, None) => None,
            };
            let axis_aligned_clips = clip_ancestors
                .iter()
                .all(|clip| clip.transform == Affine::IDENTITY.as_coeffs())
                && (widget_self_clip.is_none()
                    || css_transforms[&placed_node.node_id] == Affine::IDENTITY);
            let effective =
                axis_aligned_clips
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
            nodes.push(wabou_devtools::DebugNode {
                id,
                parent_id: placed_node
                    .parent_node_id
                    .and_then(|parent| self.node_store.node_to_solid.get(&parent).copied()),
                tag,
                text: placed_node
                    .paint
                    .text
                    .as_deref()
                    .map(|text| text.chars().take(4096).collect()),
                classes,
                matched_rules,
                style_diagnostics: self
                    .style_diagnostics
                    .get(&placed_node.node_id)
                    .cloned()
                    .unwrap_or_default(),
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
                widget: self
                    .widget_manager
                    .widgets
                    .contains_key(&placed_node.node_id)
                    .then(|| "native".into()),
                clip: wabou_devtools::DebugClipInfo {
                    widget_local,
                    chain: clip_ancestors,
                    effective,
                    static_transform: static_transform.as_coeffs(),
                    runtime_transform: placed_node
                        .paint
                        .runtime_transform
                        .map(|matrix| matrix.map(f64::from)),
                    border_transform: css_transforms[&placed_node.node_id].as_coeffs(),
                    scene_transform: content_transform.as_coeffs(),
                    device_scale: self.device_scale,
                },
                computed: wabou_devtools::DebugComputedStyle {
                    display: layout.map(|style| format!("{:?}", style.display)),
                    position: layout.map(|style| format!("{:?}", style.position)),
                    overflow_x: layout.map(|style| format!("{:?}", style.overflow.x)),
                    overflow_y: layout.map(|style| format!("{:?}", style.overflow.y)),
                    font_size: placed_node.paint.font_size,
                    font_weight: placed_node.paint.font_weight,
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
                revision: self.projections.debug_revision,
                viewport_width: self.last_viewport.0,
                viewport_height: self.last_viewport.1,
                device_scale: self.device_scale,
                node_count: nodes.len(),
                focused_node: self.input.focused_target,
                hovered_node: self.input.hovered_target,
            },
            nodes,
        };
        drop(atoms);
        if let Ok(mut state) = state.write() {
            state.publish(snapshot);
        }
    }
}
