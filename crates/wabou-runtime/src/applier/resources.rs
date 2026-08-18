use std::collections::{HashMap, HashSet};
use std::sync::{Arc, mpsc};

use taffy::NodeId;

use crate::asset_cache::{RasterAsset, ResourceCache};

pub(super) struct ImageLoadResult {
    pub(super) source: Arc<str>,
    pub(super) result: RasterAsset,
}

/// Retained image/SVG resources and their asynchronous load bookkeeping.
///
/// Keeping this state together makes source replacement and node removal one
/// lifecycle instead of a set of unrelated `Applier` maps.
pub(super) struct ResourceState {
    pub(super) svg: HashMap<NodeId, (Arc<str>, Arc<wabou_shell::svg::SvgImage>)>,
    pub(super) cache: Arc<ResourceCache>,
    pub(super) pending_images: HashSet<Arc<str>>,
    pub(super) image_subscribers: HashMap<Arc<str>, HashSet<NodeId>>,
    pub(super) node_image_sources: HashMap<NodeId, Arc<str>>,
    pub(super) result_tx: mpsc::Sender<ImageLoadResult>,
    pub(super) result_rx: mpsc::Receiver<ImageLoadResult>,
}

impl Default for ResourceState {
    fn default() -> Self {
        let (result_tx, result_rx) = mpsc::channel();
        Self {
            svg: HashMap::new(),
            cache: Arc::new(ResourceCache::memory_only()),
            pending_images: HashSet::new(),
            image_subscribers: HashMap::new(),
            node_image_sources: HashMap::new(),
            result_tx,
            result_rx,
        }
    }
}

impl ResourceState {
    pub(super) fn set_cache(&mut self, cache: Arc<ResourceCache>) {
        self.cache = cache;
    }

    pub(super) fn clear_scene_bindings(&mut self) {
        self.svg.clear();
        self.image_subscribers.clear();
        self.node_image_sources.clear();
    }
}
