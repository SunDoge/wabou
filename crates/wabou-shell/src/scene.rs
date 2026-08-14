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
        append_fragment(scene, widget, Some(transform));
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
    append_fragment(&mut clipped, widget, None);
    clipped.pop_layer();
    append_fragment(scene, &clipped, Some(transform));
}

/// Append a retained scene fragment without leaking its encoder state into
/// commands recorded afterwards.
///
/// Vello 0.9 copies the child encoding's state flags during `Scene::append`.
/// The transform and style at the append boundary therefore cannot safely be
/// deduplicated against the next command in the parent scene.
fn append_fragment(scene: &mut Scene, fragment: &Scene, transform: Option<Affine>) {
    scene.append(fragment, transform);
    scene.encoding_mut().force_next_transform_and_style();
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

fn draw_node_box(scene: &mut Scene, node: &PlacedNode, transform: Affine) {
    let [x0, y0, x1, y1] = node.rect;
    let rect = Rect::new(f64::from(x0), f64::from(y0), f64::from(x1), f64::from(y1));
    let radius = f64::from(node.paint.border_radius);
    for shadow in &node.paint.shadows {
        let (shadow_rect, radius, std_dev) = shadow_geometry(rect, radius, shadow);
        scene.draw_blurred_rounded_rect(transform, shadow_rect, shadow.color, radius, std_dev);
    }
    if let Some(background) = node.paint.background {
        scene.fill(
            Fill::NonZero,
            transform,
            background,
            None,
            &rect.to_rounded_rect(radius),
        );
    }
    let Some((_, border_color)) = node.paint.border else {
        return;
    };
    let [top, right, bottom, left] = node.border_widths;
    if top > 0.0 && top == right && top == bottom && top == left {
        let width = f64::from(top);
        let half = width / 2.0;
        let inset = Rect::new(
            f64::from(x0) + half,
            f64::from(y0) + half,
            f64::from(x1) - half,
            f64::from(y1) - half,
        );
        scene.stroke(
            &Stroke::new(width),
            transform,
            border_color,
            None,
            &inset.to_rounded_rect((radius - half).max(0.0)),
        );
        return;
    }
    // Side-specific utility borders must not turn into a uniform rectangle.
    let sides = [
        (top, (x0, y0 + top * 0.5), (x1, y0 + top * 0.5)),
        (right, (x1 - right * 0.5, y0), (x1 - right * 0.5, y1)),
        (bottom, (x0, y1 - bottom * 0.5), (x1, y1 - bottom * 0.5)),
        (left, (x0 + left * 0.5, y0), (x0 + left * 0.5, y1)),
    ];
    for (width, from, to) in sides {
        if width > 0.0 {
            let line = vello::kurbo::Line::new(
                (f64::from(from.0), f64::from(from.1)),
                (f64::from(to.0), f64::from(to.1)),
            );
            scene.stroke(
                &Stroke::new(f64::from(width)),
                transform,
                border_color,
                None,
                &line,
            );
        }
    }
}

fn draw_svg(scene: &mut Scene, node: &PlacedNode, transform: Affine) {
    let Some(svg) = &node.paint.svg else {
        return;
    };
    let [svg_width, svg_height] = svg.size();
    let [x0, y0, x1, y1] = node.rect;
    let width = (x1 - x0).max(0.0);
    let height = (y1 - y0).max(0.0);
    if svg_width <= 0.0 || svg_height <= 0.0 || width <= 0.0 || height <= 0.0 {
        return;
    }
    let scale = f64::from((width / svg_width).min(height / svg_height));
    let dx = f64::from(x0) + (f64::from(width) - f64::from(svg_width) * scale) * 0.5;
    let dy = f64::from(y0) + (f64::from(height) - f64::from(svg_height) * scale) * 0.5;
    append_fragment(
        scene,
        svg.scene(),
        Some(transform * Affine::translate((dx, dy)) * Affine::scale(scale)),
    );
}

fn draw_image(scene: &mut Scene, node: &PlacedNode, transform: Affine) {
    let Some(image) = &node.paint.image else {
        return;
    };
    let [image_width, image_height] = image.size();
    let [x0, y0, x1, y1] = node.rect;
    let width = (x1 - x0).max(0.0);
    let height = (y1 - y0).max(0.0);
    if image_width <= 0.0 || image_height <= 0.0 || width <= 0.0 || height <= 0.0 {
        return;
    }
    let scale = f64::from((width / image_width).min(height / image_height));
    let dx = f64::from(x0) + (f64::from(width) - f64::from(image_width) * scale) * 0.5;
    let dy = f64::from(y0) + (f64::from(height) - f64::from(image_height) * scale) * 0.5;
    scene.draw_image(
        image.brush(),
        transform * Affine::translate((dx, dy)) * Affine::scale(scale),
    );
}

fn draw_text(
    scene: &mut Scene,
    node: &PlacedNode,
    tcx: &mut TextContext,
    transform: Affine,
    device_scale: f64,
) {
    let Some(text) = &node.paint.text else {
        return;
    };
    let layout = crate::text::layout_text_styled_overflow(
        tcx,
        text.clone(),
        node.paint.font_size,
        node.paint.font_weight,
        node.paint.line_height,
        node.paint.text_align,
        crate::text::brush_for_color(node.paint.text_color),
        node.paint.text_runs.clone(),
        node.paint.font_family.as_ref(),
        (node.paint.wrap_text || node.paint.text_ellipsis)
            .then_some((node.rect[2] - node.rect[0]).max(0.0)),
        node.paint.text_ellipsis,
    );
    let origin = Affine::translate((
        f64::from(node.content_origin[0]),
        f64::from(node.content_origin[1]),
    ));
    for &[x0, y0, x1, y1] in node.paint.selection_rects.iter() {
        scene.fill(
            Fill::NonZero,
            transform * origin,
            Color::from_rgba8(59, 130, 246, 105),
            None,
            &Rect::new(f64::from(x0), f64::from(y0), f64::from(x1), f64::from(y1)),
        );
    }
    let glyph_scene = tcx.glyph_scene_scaled(&layout, device_scale);
    append_fragment(
        scene,
        &glyph_scene,
        Some(transform * origin * Affine::scale(device_scale.recip())),
    );
}

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

        draw_node_box(scene, n, node_transform);

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

        draw_svg(scene, n, node_transform);
        draw_image(scene, n, node_transform);

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

        draw_text(scene, n, tcx, node_transform, device_scale);
    }
    while layers.pop().is_some() {
        scene.pop_layer();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{Paint, PaintTransform, Shadow};
    use std::sync::Mutex;

    // wgpu's Linux software/EGL backend is not safe to initialize concurrently
    // in the same test process. Keep pixel tests parallel with the rest of the
    // suite while serializing only renderer creation.
    static OFFSCREEN_RENDER_LOCK: Mutex<()> = Mutex::new(());

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
        let _render_guard = OFFSCREEN_RENDER_LOCK.lock().expect("offscreen render lock");
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

    #[test]
    fn decoded_raster_image_reaches_offscreen_pixels() {
        use image::ImageEncoder as _;
        use std::sync::Arc;
        let mut encoded = Vec::new();
        image::codecs::png::PngEncoder::new(&mut encoded)
            .write_image(&[20, 180, 240, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        let raster = Arc::new(crate::image::RasterImage::decode_png(&encoded).unwrap());
        let mut node = placed_node(Paint {
            image: Some(raster),
            ..Paint::default()
        });
        node.rect = [20.0, 20.0, 80.0, 80.0];
        node.content_origin = [20.0, 20.0];
        let image = render_nodes(&[node], "raster-image");
        assert_eq!(image.get_pixel(50, 50).0, [20, 180, 240, 255]);
        assert_eq!(image.get_pixel(10, 10).0, [0, 0, 0, 255]);
    }

    fn render_nodes(nodes: &[PlacedNode], name: &str) -> image::RgbaImage {
        let _render_guard = OFFSCREEN_RENDER_LOCK.lock().expect("offscreen render lock");
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

    #[test]
    fn cached_text_fragment_does_not_corrupt_the_following_fill() {
        let mut clip = placed_node(Paint::default());
        clip.rect = [5.0, 5.0, 95.0, 95.0];
        clip.content_origin = [5.0, 5.0];
        clip.own_clip = Some(clip.rect);
        let mut nodes = vec![clip];
        for index in 0..4_u64 {
            let y = 10.0 + index as f32 * 20.0;
            let mut option = placed_node(Paint {
                background: (index == 1).then_some(Color::from_rgba8(51, 65, 85, 255)),
                ..Paint::default()
            });
            option.node_id = taffy::tree::NodeId::from(index * 2 + 1);
            option.parent_node_id = Some(nodes[0].node_id);
            option.depth = 1;
            option.rect = [10.0, y, 90.0, y + 20.0];
            option.content_origin = [10.0, y];

            let mut text = placed_node(Paint {
                text: Some("option".into()),
                text_color: Color::WHITE,
                ..Paint::default()
            });
            text.node_id = taffy::tree::NodeId::from(index * 2 + 2);
            text.parent_node_id = Some(option.node_id);
            text.depth = 2;
            text.rect = option.rect;
            text.content_origin = [15.0, y];
            nodes.extend([option, text]);
        }

        let image = render_nodes(&nodes, "text-followed-by-fill");
        assert_eq!(image.get_pixel(20, 35).0, [51, 65, 85, 255]);
    }
}
