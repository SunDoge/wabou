//! Native widget ownership and host-action routing state.

use super::*;

/// A clipboard effect completion awaiting widget delivery:
/// `(origin node, widget-local request id, clipboard text)`.
type WidgetClipboardCompletion = (NodeId, u64, Option<String>);

pub(super) struct WidgetManager {
    pub(super) widgets: HashMap<NodeId, Box<dyn wabou_shell::Widget>>,
    pub(super) styles: HashMap<NodeId, wabou_shell::WidgetStyle>,
    pub(super) geometries: HashMap<NodeId, wabou_shell::WidgetGeometry>,
    pub(super) visibility: HashMap<NodeId, bool>,
    pub(super) factories: HashMap<Atom, wabou_shell::WidgetFactory>,
    /// Completions of clipboard effects submitted by widgets, buffered until
    /// the next poll delivers them to the originating widget.
    pub(super) clipboard_completions:
        std::rc::Rc<std::cell::RefCell<Vec<WidgetClipboardCompletion>>>,
    pub(super) pending_value_sync: HashSet<NodeKey>,
    pub(super) pending_selection_sync: HashSet<NodeKey>,
}

impl WidgetManager {
    pub(super) fn new(factories: HashMap<Atom, wabou_shell::WidgetFactory>) -> Self {
        Self {
            widgets: HashMap::new(),
            styles: HashMap::new(),
            geometries: HashMap::new(),
            visibility: HashMap::new(),
            factories,
            clipboard_completions: std::rc::Rc::default(),
            pending_value_sync: HashSet::new(),
            pending_selection_sync: HashSet::new(),
        }
    }

    pub(super) fn create(
        &self,
        tag: Atom,
        wake: Option<&WakeCallback>,
        clock: wabou_host_api::SharedClock,
    ) -> Option<Box<dyn wabou_shell::Widget>> {
        self.factories.get(&tag).map(|factory| {
            let mut widget = factory();
            widget.set_clock(clock);
            if let Some(wake) = wake {
                widget.set_wake_callback(wake.clone());
            }
            widget
        })
    }
}
