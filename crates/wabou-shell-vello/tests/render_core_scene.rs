#![cfg(not(target_arch = "wasm32"))]
mod tests {
    use vello_common::kurbo::{Affine, Rect};
    use vello_common::peniko::{Color, Fill};
    use wabou_render_core::layout::PlacedNode;
    use wabou_render_core::scene::*;
    use wabou_render_core::style::{CornerRadii, IrLength};
    use wabou_render_core::style::{Paint, PaintTransform};
    use wabou_render_core::{PaintScene, Scene, TextContext};
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
            clip_radii: CornerRadii::default(),
            clip_depth: None,
            own_clip: None,
            own_clip_radius: 0.0,
            own_clip_radii: CornerRadii::default(),
            border_widths: [0.0; 4],
            scroll: wabou_render_core::layout::ScrollMetrics::default(),
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
    fn rotation_keeps_a_non_origin_border_box_center_fixed() {
        let node = placed_node(Paint {
            runtime_transform: Some(
                Affine::rotate(std::f64::consts::FRAC_PI_2)
                    .as_coeffs()
                    .map(|value| value as f32),
            ),
            ..Paint::default()
        });
        let transform = resolve_node_transform(&node, Affine::IDENTITY);
        let center = Rect::new(10.0, 20.0, 110.0, 120.0).center();
        let transformed = transform * center;
        assert!((transformed.x - center.x).abs() < 1e-9);
        assert!((transformed.y - center.y).abs() < 1e-9);
    }

    #[test]
    fn transform_origin_changes_the_static_and_runtime_pivot() {
        let node = placed_node(Paint {
            transform: vec![PaintTransform::Rotate(std::f32::consts::FRAC_PI_2)],
            transform_origin: [
                IrLength::Percent { value: 0.0 },
                IrLength::Percent { value: 0.0 },
            ],
            ..Paint::default()
        });
        let transform = resolve_node_transform(&node, Affine::IDENTITY);
        let origin = vello_common::kurbo::Point::new(10.0, 20.0);
        let transformed = transform * origin;
        assert!((transformed.x - origin.x).abs() < 1e-9);
        assert!((transformed.y - origin.y).abs() < 1e-9);
    }

    #[test]
    fn rounded_native_widget_is_clipped_without_overflow_hidden() {
        let mut widget = Scene::new();
        widget.push_clip_layer(
            Affine::IDENTITY,
            &Rect::new(0.0, 0.0, 80.0, 80.0).to_rounded_rect(12.0),
        );
        widget.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgba8(255, 0, 0, 255),
            None,
            &Rect::new(0.0, 0.0, 80.0, 80.0),
        );
        widget.pop_layer();
        let mut node = placed_node(Paint {
            border_radii: CornerRadii::uniform(12.0),
            widget: Some(std::sync::Arc::new(widget)),
            ..Paint::default()
        });
        node.rect = [10.0, 10.0, 90.0, 90.0];
        node.content_origin = [10.0, 10.0];
        node.content_size = [80.0, 80.0];

        let image = render_nodes(&[node], "rounded-native-widget");
        assert_eq!(image.get_pixel(10, 10).0, [0, 0, 0, 255]);
        assert_eq!(image.get_pixel(20, 20).0, [255, 0, 0, 255]);
    }

    #[test]
    fn node_background_preserves_individual_corner_radii() {
        let mut node = placed_node(Paint {
            background: Some(Color::from_rgba8(255, 0, 0, 255)),
            border_radii: CornerRadii {
                top_left: 0.0,
                top_right: 20.0,
                bottom_right: 0.0,
                bottom_left: 20.0,
            },
            ..Paint::default()
        });
        node.rect = [10.0, 10.0, 90.0, 90.0];

        let image = render_nodes(&[node], "individual-corner-radii");
        assert_eq!(image.get_pixel(11, 11).0, [255, 0, 0, 255]);
        assert_eq!(image.get_pixel(88, 11).0, [0, 0, 0, 255]);
        assert_eq!(image.get_pixel(88, 88).0, [255, 0, 0, 255]);
        assert_eq!(image.get_pixel(11, 88).0, [0, 0, 0, 255]);
    }

    #[test]
    fn side_specific_border_respects_the_outer_corner_radius() {
        let border = Color::from_rgba8(0, 160, 255, 255);
        let mut node = placed_node(Paint {
            border: Some((4.0, border)),
            border_radii: CornerRadii::uniform(16.0),
            ..Paint::default()
        });
        node.rect = [10.0, 10.0, 90.0, 90.0];
        node.border_widths = [4.0, 0.0, 0.0, 0.0];

        let image = render_nodes(&[node], "rounded-side-border");
        assert_eq!(
            image.get_pixel(10, 12).0,
            [0, 0, 0, 255],
            "the top border must not leave a square corner outside its radius"
        );
        assert_eq!(image.get_pixel(50, 12).0, [0, 160, 255, 255]);
    }

    #[test]
    fn overlay_scrollbar_pixels_scale_once_at_one_and_two_x() {
        let mut node = placed_node(Paint::default());
        node.rect = [0.0, 0.0, 100.0, 100.0];
        node.content_origin = [0.0, 0.0];
        node.scroll = wabou_render_core::layout::ScrollMetrics {
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
            wabou_shell_vello::renderer::render_to_png(
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
    fn transparent_scene_background_preserves_zero_alpha() {
        let mut scene = Scene::new();
        build_scene_scaled(
            &mut scene,
            &[],
            &mut TextContext::new(),
            16,
            16,
            Color::TRANSPARENT,
            1.0,
        );
        let image =
            wabou_shell_vello::renderer::render_to_image(&scene, 16, 16, Color::TRANSPARENT)
                .expect("render transparent scene");
        assert_eq!(image.get_pixel(8, 8).0, [0, 0, 0, 0]);
    }

    #[test]
    fn decoded_raster_image_reaches_offscreen_pixels() {
        use image::ImageEncoder as _;
        use std::sync::Arc;
        let mut encoded = Vec::new();
        image::codecs::png::PngEncoder::new(&mut encoded)
            .write_image(&[20, 180, 240, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        let raster = Arc::new(wabou_render_core::image::RasterImage::decode(&encoded).unwrap());
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
        wabou_shell_vello::renderer::render_to_png(
            &scene,
            100,
            100,
            Color::BLACK,
            &path.to_string_lossy(),
        )
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
        // Sample the second option outside its text. Direct outline rendering
        // legitimately covers the old (20, 35) sample with the letter `o`.
        assert_eq!(image.get_pixel(80, 35).0, [51, 65, 85, 255]);
    }

    #[test]
    fn offscreen_text_nodes_do_not_grow_the_encoded_frame() {
        let mut nodes = Vec::new();
        for index in 0..2_000_u64 {
            let y = 1_000.0 + index as f32 * 24.0;
            let mut text = placed_node(Paint {
                text: Some("a long retained conversation line that is outside the viewport".into()),
                text_color: Color::WHITE,
                ..Paint::default()
            });
            text.node_id = taffy::tree::NodeId::from(index + 1);
            text.rect = [0.0, y, 400.0, y + 20.0];
            text.content_origin = [0.0, y];
            nodes.push(text);
        }
        let mut visible = placed_node(Paint {
            text: Some("visible".into()),
            text_color: Color::WHITE,
            ..Paint::default()
        });
        visible.node_id = taffy::tree::NodeId::from(2_001_u64);
        visible.rect = [0.0, 10.0, 100.0, 30.0];
        visible.content_origin = [0.0, 10.0];
        nodes.push(visible);

        let mut scene = Scene::new();
        build_scene_scaled(
            &mut scene,
            &nodes,
            &mut TextContext::new(),
            400,
            100,
            Color::BLACK,
            1.0,
        );

        assert!(
            scene.commands.len() < 16,
            "offscreen retained text expanded to {} frame commands",
            scene.commands.len()
        );
    }

    #[test]
    fn outline_paints_outside_the_border_box_without_changing_geometry() {
        let mut node = placed_node(Paint {
            background: Some(Color::WHITE),
            outline_width: 2.0,
            outline_offset: 2.0,
            outline_color: Some(Color::from_rgba8(56, 189, 248, 255)),
            ..Paint::default()
        });
        node.rect = [20.0, 20.0, 60.0, 60.0];
        node.content_origin = [20.0, 20.0];

        let image = render_nodes(&[node], "outline-outside-border-box");
        assert_eq!(image.get_pixel(17, 30).0, [56, 189, 248, 255]);
        assert_eq!(image.get_pixel(20, 30).0, [255, 255, 255, 255]);
    }
}
