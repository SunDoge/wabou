use std::collections::HashMap;
use std::sync::Arc;

use taffy::NodeId;

use crate::asset_cache::ResourceCache;

/// Retained SVG cache plus the explicit application image registry.
///
/// Keeping this state together makes source replacement and node removal one
/// lifecycle instead of a set of unrelated `Applier` maps.
pub(super) struct ResourceState {
    pub(super) svg: HashMap<NodeId, (Arc<str>, Arc<wabou_shell::svg::SvgImage>)>,
    pub(super) cache: Arc<ResourceCache>,
    pub(super) image_store: crate::ImageResourceStore,
}

impl Default for ResourceState {
    fn default() -> Self {
        Self {
            svg: HashMap::new(),
            cache: Arc::new(ResourceCache::memory_only()),
            image_store: crate::ImageResourceStore::default(),
        }
    }
}

impl ResourceState {
    pub(super) fn set_cache(&mut self, cache: Arc<ResourceCache>) {
        self.cache = cache;
    }

    pub(super) fn set_image_store(&mut self, store: crate::ImageResourceStore) {
        self.image_store = store;
    }

    pub(super) fn clear_scene_bindings(&mut self) {
        self.svg.clear();
    }
}
