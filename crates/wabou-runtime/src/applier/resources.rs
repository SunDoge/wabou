use std::collections::HashMap;
use std::sync::Arc;

use taffy::NodeId;

type SvgAsset = Result<Arc<legacy_shell::svg::SvgImage>, Arc<str>>;

/// Retained SVG cache plus the explicit application image registry.
///
/// Keeping this state together makes source replacement and node removal one
/// lifecycle instead of a set of unrelated `Applier` maps.
#[derive(Default)]
pub(super) struct ResourceState {
    pub(super) svg: HashMap<NodeId, (Arc<str>, Arc<legacy_shell::svg::SvgImage>)>,
    pub(super) decoded_svg: HashMap<Arc<str>, SvgAsset>,
    pub(super) image_store: crate::ImageResourceStore,
}

impl ResourceState {
    pub(super) fn set_image_store(&mut self, store: crate::ImageResourceStore) {
        self.image_store = store;
    }

    pub(super) fn clear_scene_bindings(&mut self) {
        self.svg.clear();
    }
}
