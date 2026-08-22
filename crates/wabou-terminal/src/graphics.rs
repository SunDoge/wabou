use std::collections::HashMap;
use std::sync::Arc;

use rio_graphics::{ColorType, GraphicData, GraphicOverlay, atlas_image_key, kitty_image_key};
use rio_vt::ansi::graphics::{
    AtlasPlacement, KittyPlacement, OverlayViewport, UpdateQueues, atlas_overlay_geometry,
    clip_overlay_to_rect, kitty_overlay_geometry,
};
use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Blob, Fill, ImageAlphaType, ImageBrush, ImageData, ImageFormat};

#[derive(Clone)]
struct CachedGraphic {
    brush: ImageBrush,
    width: usize,
    height: usize,
}

#[derive(Default)]
pub(crate) struct TerminalGraphics {
    images: HashMap<u64, CachedGraphic>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum KittyLayer {
    BehindText,
    AboveText,
}

impl TerminalGraphics {
    #[cfg(test)]
    pub(crate) fn contains(&self, key: u64) -> bool {
        self.images.contains_key(&key)
    }

    pub(crate) fn apply_updates(&mut self, queues: UpdateQueues) {
        for key in queues.remove_queue {
            self.images.remove(&key);
        }
        for graphic in queues.pending {
            self.insert(atlas_image_key(graphic.id.get()), graphic);
        }
        for (image_id, graphic) in queues.pending_images {
            self.insert(kitty_image_key(image_id), graphic);
        }
    }

    fn insert(&mut self, key: u64, graphic: GraphicData) {
        let Some(pixels) = rgba_pixels(&graphic) else {
            tracing::warn!(
                key,
                width = graphic.width,
                height = graphic.height,
                "ignored malformed terminal graphic"
            );
            return;
        };
        let data = ImageData {
            data: Blob::new(Arc::new(pixels.into_boxed_slice())),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width: graphic.width as u32,
            height: graphic.height as u32,
        };
        self.images.insert(
            key,
            CachedGraphic {
                brush: ImageBrush::new(data),
                width: graphic.width,
                height: graphic.height,
            },
        );
    }

    pub(crate) fn draw_atlas(
        &self,
        scene: &mut Scene,
        placements: &[AtlasPlacement],
        viewport: &OverlayViewport,
        clip: [f32; 4],
        device_scale: f64,
    ) {
        for placement in placements {
            let Some(geometry) = atlas_overlay_geometry(placement, viewport) else {
                continue;
            };
            let overlay = GraphicOverlay {
                image_id: placement.image_key,
                x: geometry.x,
                y: geometry.y,
                width: geometry.width,
                height: geometry.height,
                z_index: -1,
                source_rect: geometry.source_rect,
            };
            self.draw_overlay(scene, overlay, clip, device_scale);
        }
    }

    pub(crate) fn draw_kitty(
        &self,
        scene: &mut Scene,
        placements: &[KittyPlacement],
        layer: KittyLayer,
        viewport: &OverlayViewport,
        clip: [f32; 4],
        device_scale: f64,
    ) {
        let mut placements: Vec<_> = placements
            .iter()
            .filter(|placement| match layer {
                KittyLayer::BehindText => placement.z_index < 0,
                KittyLayer::AboveText => placement.z_index >= 0,
            })
            .collect();
        placements.sort_unstable_by_key(|p| (p.z_index, p.image_id, p.placement_id));
        for placement in placements {
            let key = kitty_image_key(placement.image_id);
            let Some(image) = self.images.get(&key) else {
                continue;
            };
            let Some(geometry) =
                kitty_overlay_geometry(placement, image.width, image.height, viewport)
            else {
                continue;
            };
            let overlay = GraphicOverlay {
                image_id: key,
                x: geometry.x,
                y: geometry.y,
                width: geometry.width,
                height: geometry.height,
                z_index: placement.z_index,
                source_rect: geometry.source_rect,
            };
            self.draw_overlay(scene, overlay, clip, device_scale);
        }
    }

    fn draw_overlay(
        &self,
        scene: &mut Scene,
        mut overlay: GraphicOverlay,
        clip: [f32; 4],
        device_scale: f64,
    ) {
        if !clip_overlay_to_rect(&mut overlay, clip[0], clip[1], clip[2], clip[3]) {
            return;
        }
        let Some(image) = self.images.get(&overlay.image_id) else {
            return;
        };
        let scale = device_scale.max(f64::EPSILON);
        let Some((destination, transform)) =
            overlay_transform(&overlay, image.width, image.height, scale)
        else {
            return;
        };
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &destination);
        scene.draw_image(&image.brush, transform);
        scene.pop_layer();
    }
}

fn overlay_transform(
    overlay: &GraphicOverlay,
    image_width: usize,
    image_height: usize,
    device_scale: f64,
) -> Option<(Rect, Affine)> {
    let destination = Rect::new(
        overlay.x as f64 / device_scale,
        overlay.y as f64 / device_scale,
        (overlay.x + overlay.width) as f64 / device_scale,
        (overlay.y + overlay.height) as f64 / device_scale,
    );
    let [u0, v0, u1, v1] = overlay.source_rect;
    let source_x = f64::from(u0) * image_width as f64;
    let source_y = f64::from(v0) * image_height as f64;
    let source_width = f64::from(u1 - u0) * image_width as f64;
    let source_height = f64::from(v1 - v0) * image_height as f64;
    if source_width <= 0.0 || source_height <= 0.0 {
        return None;
    }
    let sx = destination.width() / source_width;
    let sy = destination.height() / source_height;
    Some((
        destination,
        Affine::translate((
            destination.x0 - source_x * sx,
            destination.y0 - source_y * sy,
        )) * Affine::scale_non_uniform(sx, sy),
    ))
}

fn rgba_pixels(graphic: &GraphicData) -> Option<Vec<u8>> {
    let pixels = graphic.width.checked_mul(graphic.height)?;
    match graphic.color_type {
        ColorType::Rgba if graphic.pixels.len() == pixels.checked_mul(4)? => {
            Some(graphic.pixels.clone())
        }
        ColorType::Rgb if graphic.pixels.len() == pixels.checked_mul(3)? => {
            let mut rgba = Vec::with_capacity(pixels * 4);
            for rgb in graphic.pixels.as_chunks::<3>().0 {
                rgba.extend_from_slice(rgb);
                rgba.push(255);
            }
            Some(rgba)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use rio_graphics::{GraphicId, ResizeCommand};

    use super::*;

    fn graphic(color_type: ColorType, pixels: Vec<u8>) -> GraphicData {
        GraphicData {
            id: GraphicId::new(7),
            width: 2,
            height: 1,
            color_type,
            pixels,
            is_opaque: true,
            resize: None::<ResizeCommand>,
            display_width: None,
            display_height: None,
            transmit_time: std::time::Instant::now(),
        }
    }

    #[test]
    fn rgb_is_expanded_and_malformed_graphics_are_rejected() {
        assert_eq!(
            rgba_pixels(&graphic(ColorType::Rgb, vec![1, 2, 3, 4, 5, 6])),
            Some(vec![1, 2, 3, 255, 4, 5, 6, 255])
        );
        assert_eq!(
            rgba_pixels(&graphic(ColorType::Rgba, vec![1, 2, 3, 4, 5, 6, 7, 8])),
            Some(vec![1, 2, 3, 4, 5, 6, 7, 8])
        );
        assert_eq!(rgba_pixels(&graphic(ColorType::Rgb, vec![1, 2])), None);
    }

    #[test]
    fn update_queues_share_one_cache_and_honor_removals() {
        let mut cache = TerminalGraphics::default();
        let atlas_key = atlas_image_key(7);
        let kitty_key = kitty_image_key(42);
        cache.apply_updates(UpdateQueues {
            pending: vec![graphic(ColorType::Rgb, vec![1, 2, 3, 4, 5, 6])],
            pending_images: vec![(42, graphic(ColorType::Rgba, vec![1, 2, 3, 4, 5, 6, 7, 8]))],
            remove_queue: Vec::new(),
        });
        assert!(cache.images.contains_key(&atlas_key));
        assert!(cache.images.contains_key(&kitty_key));

        cache.apply_updates(UpdateQueues {
            pending: Vec::new(),
            pending_images: Vec::new(),
            remove_queue: vec![atlas_key, kitty_key],
        });
        assert!(cache.images.is_empty());
    }

    #[test]
    fn crop_transform_maps_physical_overlay_to_logical_destination() {
        let overlay = GraphicOverlay {
            image_id: 1,
            x: 20.0,
            y: 40.0,
            width: 100.0,
            height: 80.0,
            z_index: 0,
            source_rect: [0.25, 0.25, 0.75, 0.75],
        };
        let (destination, transform) = overlay_transform(&overlay, 200, 100, 2.0).unwrap();
        assert_eq!(destination, Rect::new(10.0, 20.0, 60.0, 60.0));
        assert_eq!(transform.as_coeffs(), [0.5, 0.0, 0.0, 0.8, -15.0, 0.0]);
    }
}
