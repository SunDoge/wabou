//! Native widget ownership and host-action routing state.

use super::*;

pub(super) struct WidgetManager {
    pub(super) widgets: HashMap<NodeId, Box<dyn legacy_shell::Widget>>,
    pub(super) styles: HashMap<NodeId, legacy_shell::WidgetStyle>,
    pub(super) geometries: HashMap<NodeId, legacy_shell::WidgetGeometry>,
    pub(super) visibility: HashMap<NodeId, bool>,
    pub(super) factories: HashMap<Atom, legacy_shell::WidgetFactory>,
    pub(super) host_action_routes: HashMap<u64, (NodeId, u64)>,
    pub(super) next_host_action_id: u64,
    pub(super) pending_value_sync: HashSet<NodeKey>,
    pub(super) pending_selection_sync: HashSet<NodeKey>,
}

impl WidgetManager {
    pub(super) fn new(factories: HashMap<Atom, legacy_shell::WidgetFactory>) -> Self {
        Self {
            widgets: HashMap::new(),
            styles: HashMap::new(),
            geometries: HashMap::new(),
            visibility: HashMap::new(),
            factories,
            host_action_routes: HashMap::new(),
            next_host_action_id: 1,
            pending_value_sync: HashSet::new(),
            pending_selection_sync: HashSet::new(),
        }
    }

    pub(super) fn create(
        &self,
        tag: Atom,
        wake: Option<&WakeCallback>,
    ) -> Option<Box<dyn legacy_shell::Widget>> {
        self.factories.get(&tag).map(|factory| {
            let mut widget = factory();
            if let Some(wake) = wake {
                widget.set_wake_callback(wake.clone());
            }
            widget
        })
    }
}
