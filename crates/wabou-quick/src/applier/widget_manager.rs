//! Native widget ownership and host-action routing state.

use super::*;

pub(super) struct WidgetManager {
    pub(super) widgets: HashMap<NodeId, Box<dyn crate::widget::Widget>>,
    pub(super) styles: HashMap<NodeId, crate::widget::WidgetStyle>,
    pub(super) factories: HashMap<Atom, crate::widget::WidgetFactory>,
    pub(super) host_action_routes: HashMap<u64, (NodeId, u64)>,
    pub(super) next_host_action_id: u64,
    pub(super) pending_value_sync: HashSet<u32>,
}

impl WidgetManager {
    pub(super) fn new(factories: HashMap<Atom, crate::widget::WidgetFactory>) -> Self {
        Self {
            widgets: HashMap::new(),
            styles: HashMap::new(),
            factories,
            host_action_routes: HashMap::new(),
            next_host_action_id: 1,
            pending_value_sync: HashSet::new(),
        }
    }

    pub(super) fn create(
        &self,
        tag: Atom,
        wake: Option<&WakeCallback>,
    ) -> Option<Box<dyn crate::widget::Widget>> {
        self.factories.get(&tag).map(|factory| {
            let mut widget = factory();
            if let Some(wake) = wake {
                widget.set_wake_callback(wake.clone());
            }
            widget
        })
    }
}
