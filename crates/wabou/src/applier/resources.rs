use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use taffy::NodeId;

type SvgAsset = Result<Arc<wabou_shell::svg::SvgImage>, Arc<str>>;
const DECODED_SVG_CACHE_CAPACITY: usize = 256;

/// Retained SVG cache plus the explicit application image registry.
///
/// Keeping this state together makes source replacement and node removal one
/// lifecycle instead of a set of unrelated `Applier` maps.
pub(super) struct ResourceState {
    pub(super) svg: HashMap<NodeId, (Arc<str>, Arc<wabou_shell::svg::SvgImage>)>,
    pub(super) decoded_svg: LruCache<Arc<str>, SvgAsset>,
    pub(super) image_store: crate::ImageResourceStore,
}

impl Default for ResourceState {
    fn default() -> Self {
        Self {
            svg: HashMap::new(),
            decoded_svg: LruCache::new(
                NonZeroUsize::new(DECODED_SVG_CACHE_CAPACITY).expect("non-zero SVG cache"),
            ),
            image_store: crate::ImageResourceStore::default(),
        }
    }
}

impl ResourceState {
    pub(super) fn set_image_store(&mut self, store: crate::ImageResourceStore) {
        self.image_store = store;
    }

    pub(super) fn clear_scene_bindings(&mut self) {
        self.svg.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_svg_cache_evicts_old_pages() {
        let mut resources = ResourceState::default();
        for index in 0..=DECODED_SVG_CACHE_CAPACITY {
            let source: Arc<str> = Arc::from(format!("<svg id='{index}'/>"));
            resources
                .decoded_svg
                .put(source, Err(Arc::from("fixture parse error")));
        }

        assert_eq!(resources.decoded_svg.len(), DECODED_SVG_CACHE_CAPACITY);
        assert!(resources.decoded_svg.peek("<svg id='0'/>").is_none());
        assert!(resources.decoded_svg.peek("<svg id='256'/>").is_some());
    }
}
