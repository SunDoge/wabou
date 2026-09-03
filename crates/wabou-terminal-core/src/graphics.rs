use std::collections::HashMap;
use std::sync::Arc;

use rio_graphics::{ColorType, GraphicData, atlas_image_key, kitty_image_key};
use rio_vt::ansi::graphics::UpdateQueues;

#[derive(Clone)]
pub(super) struct CachedGraphic {
    pub(super) pixels: Arc<Vec<u8>>,
    pub(super) width: usize,
    pub(super) height: usize,
}

/// Renderer-neutral decoded terminal image resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalImage {
    pub key: u64,
    pub pixels: Arc<Vec<u8>>,
    pub width: usize,
    pub height: usize,
}

#[derive(Default)]
pub(crate) struct TerminalGraphics {
    pub(super) images: HashMap<u64, CachedGraphic>,
}

impl TerminalGraphics {
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

    pub(crate) fn snapshot(&self) -> Vec<TerminalImage> {
        self.images
            .iter()
            .map(|(&key, image)| TerminalImage {
                key,
                pixels: image.pixels.clone(),
                width: image.width,
                height: image.height,
            })
            .collect()
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
        self.images.insert(
            key,
            CachedGraphic {
                pixels: Arc::new(pixels),
                width: graphic.width,
                height: graphic.height,
            },
        );
    }
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
}
