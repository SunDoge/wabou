use anyrender::{PaintScene, Scene};
use rio_graphics::{GraphicOverlay, kitty_image_key};
use rio_vt::ansi::graphics::{
    AtlasPlacement, KittyPlacement, OverlayViewport, atlas_overlay_geometry, clip_overlay_to_rect,
    kitty_overlay_geometry,
};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};

use crate::graphics::TerminalGraphics;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum KittyLayer {
    BehindText,
    AboveText,
}

impl TerminalGraphics {
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
        let brush = ImageBrush::new(ImageData {
            data: Blob::new(image.pixels.clone()),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width: image.width as u32,
            height: image.height as u32,
        });
        let scale = device_scale.max(f64::EPSILON);
        let Some((destination, transform)) =
            overlay_transform(&overlay, image.width, image.height, scale)
        else {
            return;
        };
        scene.push_clip_layer(Affine::IDENTITY, &destination);
        scene.draw_image((&brush).into(), transform);
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

#[cfg(test)]
mod tests {
    use super::*;

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
