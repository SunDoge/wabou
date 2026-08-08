//! Build a `vello::Scene` from the flattened layout list.

use std::collections::HashMap;

use vello::Scene;
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{Color, Fill};

use crate::layout::{PlacedNode, SubtreeEvent, subtree_events};
use crate::scrollbar::{ScrollAxis, thumb as scrollbar_thumb, track as scrollbar_track};
use crate::style::{IrLength, PaintTransform, Shadow};
use crate::text::TextContext;

/// Resolve the node-local static CSS and runtime affine transforms separately.
pub fn resolve_local_transforms(node: &PlacedNode) -> (Affine, Affine) {
    let [x0, y0, x1, y1] = node.rect;
    let resolve = |length: &IrLength, size: f32| match length {
        IrLength::Px { value } => *value as f64,
        IrLength::Percent { value } => (*value * size) as f64,
        IrLength::Auto => 0.0,
    };
    let mut static_transform = Affine::IDENTITY;
    for transform in &node.paint.transform {
        static_transform *= match transform {
            PaintTransform::Translate(x, y) => {
                Affine::translate((resolve(x, x1 - x0), resolve(y, y1 - y0)))
            }
            PaintTransform::Scale(x, y) => Affine::scale_non_uniform(*x as f64, *y as f64),
            PaintTransform::Rotate(angle) => Affine::rotate(*angle as f64),
            PaintTransform::Skew(x, y) => Affine::skew(*x as f64, *y as f64),
            PaintTransform::Matrix(matrix) => Affine::new(matrix.map(f64::from)),
        };
    }
    let runtime_transform = node
        .paint
        .runtime_transform
        .map_or(Affine::IDENTITY, |matrix| {
            Affine::new(matrix.map(f64::from))
        });
    (static_transform, runtime_transform)
}

/// Resolve static CSS and host-driven runtime state into window coordinates.
pub fn resolve_node_transform(node: &PlacedNode, parent_transform: Affine) -> Affine {
    let [x0, y0, x1, y1] = node.rect;
    let rect = Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64);
    let (static_transform, runtime_transform) = resolve_local_transforms(node);
    let center = rect.center().to_vec2();
    parent_transform
        * Affine::translate(center)
        * static_transform
        * runtime_transform
        * Affine::translate(-center)
}

fn widget_clip(node: &PlacedNode) -> Option<([f32; 4], f64)> {
    let [x0, y0, x1, y1] = node.rect;
    let [top, right, bottom, left] = node.border_widths;
    let radius = node.paint.border_radius as f64;
    if radius > 0.0 {
        let border_inset = top.max(right).max(bottom).max(left);
        return Some((
            [x0 + left, y0 + top, x1 - right, y1 - bottom],
            (radius - border_inset as f64).max(0.0),
        ));
    }
    node.own_clip
        .map(|clip| (clip, node.own_clip_radius as f64))
}

fn append_widget(scene: &mut Scene, node: &PlacedNode, widget: &Scene, transform: Affine) {
    let radius = node.paint.border_radius as f64;
    if radius <= 0.0 {
        scene.append(widget, Some(transform));
        return;
    }
    let [top, right, bottom, left] = node.border_widths;
    let inner_radius = (radius - top.max(right).max(bottom).max(left) as f64).max(0.0);
    let [width, height] = node.content_size;
    let mut clipped = Scene::new();
    clipped.push_clip_layer(
        Fill::NonZero,
        Affine::IDENTITY,
        &Rect::new(0.0, 0.0, width as f64, height as f64).to_rounded_rect(inner_radius),
    );
    clipped.append(widget, None);
    clipped.pop_layer();
    scene.append(&clipped, Some(transform));
}

fn shadow_geometry(rect: Rect, node_radius: f64, shadow: &Shadow) -> (Rect, f64, f64) {
    let spread = f64::from(shadow.spread);
    let shadow_rect = Rect::new(
        rect.x0 + f64::from(shadow.offset_x) - spread,
        rect.y0 + f64::from(shadow.offset_y) - spread,
        rect.x1 + f64::from(shadow.offset_x) + spread,
        rect.y1 + f64::from(shadow.offset_y) + spread,
    );
    let radius = shadow
        .radius
        .map(f64::from)
        .unwrap_or_else(|| (node_radius + spread).max(0.0));
    (shadow_rect, radius, f64::from(shadow.std_dev))
}

fn draw_scrollbars(scene: &mut Scene, node: &PlacedNode, transform: Affine) {
    if node.scroll.opacity <= 0.0 {
        return;
    }
    let fade = |color: Color| {
        let rgba = color.to_rgba8();
        Color::from_rgba8(
            rgba.r,
            rgba.g,
            rgba.b,
            ((rgba.a as f32) * node.scroll.opacity) as u8,
        )
    };
    for axis in [ScrollAxis::Vertical, ScrollAxis::Horizontal] {
        if let Some(track) = scrollbar_track(node, axis) {
            scene.fill(
                Fill::NonZero,
                transform,
                fade(node.paint.scrollbar.track_color),
                None,
                &track,
            );
        }
        let Some(rect) = scrollbar_thumb(node, axis) else {
            continue;
        };
        let radius = if node.paint.scrollbar.radius < 0.0 {
            match axis {
                ScrollAxis::Horizontal => rect.height() * 0.5,
                ScrollAxis::Vertical => rect.width() * 0.5,
            }
        } else {
            f64::from(node.paint.scrollbar.radius)
        };
        let color = match node.scroll.interaction {
            2 => node.paint.scrollbar.active_color,
            1 => node.paint.scrollbar.hover_color,
            _ => node.paint.scrollbar.thumb_color,
        };
        scene.fill(
            Fill::NonZero,
            transform,
            fade(color),
            None,
            &rect.to_rounded_rect(radius),
        );
    }
}

/// Paint `nodes` into `scene` over a `base_color` background. Text nodes are
/// laid out with parley and rendered as vello glyph runs.
pub fn build_scene(
    scene: &mut Scene,
    nodes: &[PlacedNode],
    tcx: &mut TextContext,
    width: u32,
    height: u32,
    base_color: Color,
) {
    build_scene_scaled(scene, nodes, tcx, width, height, base_color, 1.0);
}

/// Build a logical-pixel scene and transform it to physical pixels at encode
/// time. Layout, hit testing and font sizes stay in CSS-like logical units.
pub fn build_scene_scaled(
    scene: &mut Scene,
    nodes: &[PlacedNode],
    tcx: &mut TextContext,
    width: u32,
    height: u32,
    base_color: Color,
    device_scale: f64,
) {
    scene.reset();
    let device = Affine::scale(device_scale);

    let bg = Rect::new(0.0, 0.0, width as f64, height as f64);
    scene.fill(Fill::NonZero, device, base_color, None, &bg);

    let mut transforms = HashMap::new();
    enum Layer {
        Clip { depth: usize },
        Opacity { depth: usize },
    }
    impl Layer {
        fn depth(&self) -> usize {
            match self {
                Self::Clip { depth } | Self::Opacity { depth } => *depth,
            }
        }
    }
    let mut layers = Vec::new();
    for event in subtree_events(nodes) {
        let SubtreeEvent::Enter(n) = event else {
            let SubtreeEvent::Exit(n) = event else {
                unreachable!()
            };
            if let Some(transform) = transforms.get(&n.node_id).copied() {
                draw_scrollbars(scene, n, device * transform);
            }
            while layers
                .last()
                .is_some_and(|layer: &Layer| layer.depth() >= n.depth)
            {
                scene.pop_layer();
                layers.pop();
            }
            continue;
        };
        let [x0, y0, x1, y1] = n.rect;
        let rect = Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64);
        let r = n.paint.border_radius as f64;
        let parent_transform = n
            .parent_node_id
            .and_then(|parent| transforms.get(&parent).copied())
            .unwrap_or(Affine::IDENTITY);
        let css_transform = resolve_node_transform(n, parent_transform);
        transforms.insert(n.node_id, css_transform);
        let node_transform = device * css_transform;

        // Do not cull an individual retained node here. Its opacity and clip
        // layers apply to descendants, and a transformed descendant can still
        // enter the viewport even when this node's own border box is outside.
        // Subtree culling requires a conservative visual-subtree bound.
        if n.paint.opacity < 1.0 {
            scene.push_layer(
                Fill::NonZero,
                vello::peniko::Mix::Normal,
                n.paint.opacity,
                device,
                &bg,
            );
            layers.push(Layer::Opacity { depth: n.depth });
        }

        for shadow in &n.paint.shadows {
            let (shadow_rect, radius, std_dev) = shadow_geometry(rect, r, shadow);
            scene.draw_blurred_rounded_rect(
                node_transform,
                shadow_rect,
                shadow.color,
                radius,
                std_dev,
            );
        }

        if let Some(bg) = n.paint.background {
            let rr = rect.to_rounded_rect(r);
            scene.fill(Fill::NonZero, node_transform, bg, None, &rr);
        }

        if let Some((_, bc)) = n.paint.border {
            let [top, right, bottom, left] = n.border_widths;
            if top > 0.0 && top == right && top == bottom && top == left {
                let bw = top as f64;
                let half = bw / 2.0;
                let inset = Rect::new(
                    x0 as f64 + half,
                    y0 as f64 + half,
                    x1 as f64 - half,
                    y1 as f64 - half,
                );
                let ir = (r - half).max(0.0);
                scene.stroke(
                    &Stroke::new(bw),
                    node_transform,
                    bc,
                    None,
                    &inset.to_rounded_rect(ir),
                );
            } else {
                // Side-specific utility borders (`border-b`, `border-r`, ...)
                // must not turn into a uniform rectangle.
                let sides = [
                    (top, (x0, y0 + top * 0.5), (x1, y0 + top * 0.5)),
                    (right, (x1 - right * 0.5, y0), (x1 - right * 0.5, y1)),
                    (bottom, (x0, y1 - bottom * 0.5), (x1, y1 - bottom * 0.5)),
                    (left, (x0 + left * 0.5, y0), (x0 + left * 0.5, y1)),
                ];
                for (width, from, to) in sides {
                    if width > 0.0 {
                        let line = vello::kurbo::Line::new(
                            (from.0 as f64, from.1 as f64),
                            (to.0 as f64, to.1 as f64),
                        );
                        scene.stroke(&Stroke::new(width as f64), node_transform, bc, None, &line);
                    }
                }
            }
        }

        if let Some([cx0, cy0, cx1, cy1]) = n.own_clip {
            let extent = f64::from(width.max(height)) * 4.0 + 4096.0;
            let finite = |value: f32| {
                if value == f32::NEG_INFINITY {
                    -extent
                } else if value == f32::INFINITY {
                    extent
                } else {
                    f64::from(value)
                }
            };
            let clip_rect = Rect::new(finite(cx0), finite(cy0), finite(cx1), finite(cy1));
            scene.push_clip_layer(
                Fill::NonZero,
                node_transform,
                &clip_rect.to_rounded_rect(n.own_clip_radius as f64),
            );
            layers.push(Layer::Clip { depth: n.depth });
        }

        if let Some(svg) = &n.paint.svg {
            let [sw, sh] = svg.size();
            let width = (x1 - x0).max(0.0);
            let height = (y1 - y0).max(0.0);
            if sw > 0.0 && sh > 0.0 && width > 0.0 && height > 0.0 {
                // SVG's default preserveAspectRatio is xMidYMid meet. usvg has
                // already applied the viewBox transform within the fragment;
                // this transform fits its viewport into the CSS border box.
                let scale = (width / sw).min(height / sh) as f64;
                let dx = x0 as f64 + (width as f64 - sw as f64 * scale) * 0.5;
                let dy = y0 as f64 + (height as f64 - sh as f64 * scale) * 0.5;
                let transform = node_transform * Affine::translate((dx, dy)) * Affine::scale(scale);
                scene.append(svg.scene(), Some(transform));
            }
        }

        if let Some(ws) = &n.paint.widget {
            // Keep rounded widget clipping inside the fragment itself. Some
            // GPU backends do not reliably carry a parent clip across an
            // appended scene at HiDPI; local encoding also avoids mixing
            // absolute logical and fragment coordinates.
            let outer_widget_clip = (r <= 0.0).then(|| widget_clip(n)).flatten();
            if let Some(([cx0, cy0, cx1, cy1], radius)) = outer_widget_clip {
                let widget_clip = Rect::new(
                    cx0.max(0.0) as f64,
                    cy0.max(0.0) as f64,
                    cx1.min(width as f32) as f64,
                    cy1.min(height as f32) as f64,
                );
                scene.push_clip_layer(
                    Fill::NonZero,
                    node_transform,
                    &widget_clip.to_rounded_rect(radius),
                );
            }
            append_widget(
                scene,
                n,
                ws,
                node_transform
                    * Affine::translate((n.content_origin[0] as f64, n.content_origin[1] as f64)),
            );
            if outer_widget_clip.is_some() {
                scene.pop_layer();
            }
        }

        if let Some(text) = &n.paint.text {
            let layout = crate::text::layout_text_styled_overflow(
                tcx,
                text.clone(),
                n.paint.font_size,
                n.paint.font_weight,
                n.paint.line_height,
                n.paint.text_align,
                crate::text::brush_for_color(n.paint.text_color),
                n.paint.text_runs.clone(),
                n.paint.font_family.as_ref(),
                (n.paint.wrap_text || n.paint.text_ellipsis)
                    .then_some((n.rect[2] - n.rect[0]).max(0.0)),
                n.paint.text_ellipsis,
            );
            let text_transform = node_transform
                * Affine::translate((n.content_origin[0] as f64, n.content_origin[1] as f64));
            for &[x0, y0, x1, y1] in n.paint.selection_rects.iter() {
                scene.fill(
                    Fill::NonZero,
                    text_transform,
                    Color::from_rgba8(59, 130, 246, 105),
                    None,
                    &Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64),
                );
            }
            let glyph_scene = tcx.glyph_scene_scaled(&layout, device_scale);
            scene.append(
                &glyph_scene,
                Some(
                    node_transform
                        * Affine::translate((
                            n.content_origin[0] as f64,
                            n.content_origin[1] as f64,
                        ))
                        * Affine::scale(device_scale.recip()),
                ),
            );
        }
    }
    while layers.pop().is_some() {
        scene.pop_layer();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{Paint, PaintTransform, Shadow};

    fn placed_node(paint: Paint) -> PlacedNode {
        PlacedNode {
            node_id: taffy::tree::NodeId::from(0_u64),
            parent_node_id: None,
            depth: 0,
            rect: [10.0, 20.0, 110.0, 120.0],
            content_origin: [10.0, 20.0],
            content_size: [100.0, 100.0],
            clip: None,
            clip_radius: 0.0,
            clip_depth: None,
            own_clip: None,
            own_clip_radius: 0.0,
            border_widths: [0.0; 4],
            scroll: crate::layout::ScrollMetrics::default(),
            paint,
        }
    }

    #[test]
    fn runtime_transform_composes_after_static_css_transform() {
        let node = placed_node(Paint {
            transform: vec![PaintTransform::Scale(2.0, 2.0)],
            runtime_transform: Some([1.0, 0.0, 0.0, 1.0, 5.0, 7.0]),
            ..Paint::default()
        });
        let actual = resolve_node_transform(&node, Affine::IDENTITY);
        let center = Rect::new(10.0, 20.0, 110.0, 120.0).center().to_vec2();
        let expected = Affine::translate(center)
            * Affine::scale(2.0)
            * Affine::translate((5.0, 7.0))
            * Affine::translate(-center);
        assert_eq!(actual.as_coeffs(), expected.as_coeffs());
    }

    #[test]
    fn rounded_native_widget_is_clipped_without_overflow_hidden() {
        let node = placed_node(Paint {
            border_radius: 12.0,
            ..Paint::default()
        });

        assert_eq!(widget_clip(&node), Some(([10.0, 20.0, 110.0, 120.0], 12.0)));
    }

    #[test]
    fn shadow_geometry_preserves_vello_parameters() {
        let shadow = Shadow {
            offset_x: 6.0,
            offset_y: -4.0,
            spread: 3.0,
            std_dev: 7.5,
            color: Color::BLACK,
            radius: None,
        };
        let (rect, radius, std_dev) =
            shadow_geometry(Rect::new(10.0, 20.0, 110.0, 120.0), 8.0, &shadow);

        assert_eq!(rect, Rect::new(13.0, 13.0, 119.0, 119.0));
        assert_eq!(radius, 11.0);
        assert_eq!(std_dev, 7.5);
    }

    #[test]
    fn shadow_radius_can_be_independent_from_node_radius_and_spread() {
        let shadow = Shadow {
            offset_x: 0.0,
            offset_y: 0.0,
            spread: -20.0,
            std_dev: 2.0,
            color: Color::BLACK,
            radius: Some(24.0),
        };
        let (_, radius, _) = shadow_geometry(Rect::new(0.0, 0.0, 100.0, 100.0), 8.0, &shadow);

        assert_eq!(radius, 24.0);
    }

    #[test]
    fn overlay_scrollbar_pixels_scale_once_at_one_and_two_x() {
        let mut node = placed_node(Paint::default());
        node.rect = [0.0, 0.0, 100.0, 100.0];
        node.content_origin = [0.0, 0.0];
        node.scroll = crate::layout::ScrollMetrics {
            port: [0.0, 0.0, 100.0, 100.0],
            scrollable: [false, true],
            range: [0.0, 900.0],
            offset: [0.0, 0.0],
            opacity: 1.0,
            interaction: 0,
        };
        node.paint.scrollbar.thumb_color = Color::from_rgba8(56, 189, 248, 255);
        let mut tcx = TextContext::new();
        for scale in [1_u32, 2] {
            let mut scene = Scene::new();
            build_scene_scaled(
                &mut scene,
                std::slice::from_ref(&node),
                &mut tcx,
                100,
                100,
                Color::BLACK,
                f64::from(scale),
            );
            let path = std::env::temp_dir().join(format!(
                "wabou-scrollbar-pixel-{}-{scale}.png",
                std::process::id()
            ));
            crate::renderer::render_to_png(
                &scene,
                100 * scale,
                100 * scale,
                Color::BLACK,
                &path.to_string_lossy(),
            )
            .expect("offscreen scrollbar render");
            let image = image::open(&path).expect("rendered png").into_rgba8();
            let thumb = image.get_pixel(93 * scale, 16 * scale).0;
            assert!(thumb[2] > 200 && thumb[1] > 150, "thumb pixel: {thumb:?}");
            assert_eq!(image.get_pixel(50 * scale, 50 * scale).0[..3], [0, 0, 0]);
            std::fs::remove_file(path).expect("remove owned test png");
        }
    }

    fn render_nodes(nodes: &[PlacedNode], name: &str) -> image::RgbaImage {
        let mut scene = Scene::new();
        build_scene_scaled(
            &mut scene,
            nodes,
            &mut TextContext::new(),
            100,
            100,
            Color::BLACK,
            1.0,
        );
        let path =
            std::env::temp_dir().join(format!("wabou-scene-{name}-{}.png", std::process::id()));
        crate::renderer::render_to_png(&scene, 100, 100, Color::BLACK, &path.to_string_lossy())
            .expect("offscreen scene render");
        let image = image::open(&path).expect("rendered png").into_rgba8();
        std::fs::remove_file(path).expect("remove owned test png");
        image
    }

    fn transformed_child(parent: &PlacedNode) -> PlacedNode {
        let mut child = placed_node(Paint {
            background: Some(Color::from_rgba8(255, 0, 0, 255)),
            runtime_transform: Some([1.0, 0.0, 0.0, 1.0, 120.0, 0.0]),
            ..Paint::default()
        });
        child.node_id = taffy::tree::NodeId::from(1_u64);
        child.parent_node_id = Some(parent.node_id);
        child.depth = 1;
        child.rect = [-100.0, 10.0, -50.0, 60.0];
        child.content_origin = [-100.0, 10.0];
        child
    }

    #[test]
    fn offscreen_parent_clip_still_applies_to_transformed_descendant() {
        let mut parent = placed_node(Paint::default());
        parent.rect = [-100.0, 10.0, -50.0, 60.0];
        parent.content_origin = [-100.0, 10.0];
        parent.own_clip = Some(parent.rect);
        let child = transformed_child(&parent);

        let image = render_nodes(&[parent, child], "offscreen-parent-clip");
        assert_eq!(image.get_pixel(30, 30).0[..3], [0, 0, 0]);
    }

    #[test]
    fn offscreen_parent_opacity_still_applies_to_transformed_descendant() {
        let mut parent = placed_node(Paint {
            opacity: 0.5,
            ..Paint::default()
        });
        parent.rect = [-100.0, 10.0, -50.0, 60.0];
        parent.content_origin = [-100.0, 10.0];
        let child = transformed_child(&parent);

        let image = render_nodes(&[parent, child], "offscreen-parent-opacity");
        let pixel = image.get_pixel(30, 30).0;
        assert!(
            (120..=136).contains(&pixel[0]) && pixel[1] == 0 && pixel[2] == 0,
            "expected half-opacity red, got {pixel:?}"
        );
    }
}
