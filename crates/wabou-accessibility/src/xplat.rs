//! Window-level AccessKit adapter and retained accessibility tree publication.

use std::sync::{Arc, Mutex};

use accesskit::{
    Action, AriaCurrent, HasPopup, Node, NodeId, Rect, Role, Toggled, Tree, TreeId, TreeUpdate,
};
use accesskit_xplat::{Adapter, EventHandler, WindowEvent as AccessKitEvent};
use raw_window_handle::HasWindowHandle;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::{
    SemanticAction, SemanticCurrent, SemanticNode, SemanticPopup, SemanticRole, SemanticSnapshot,
    SemanticToggleState,
};

const ROOT_ID: NodeId = NodeId(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PublicationKey {
    semantic_revision: u64,
    width: u32,
    height: u32,
    scale_bits: u64,
}

fn publication_key(
    snapshot: Option<&SemanticSnapshot>,
    width: u32,
    height: u32,
    scale: f64,
) -> PublicationKey {
    PublicationKey {
        semantic_revision: snapshot.map_or(0, |snapshot| snapshot.revision),
        width,
        height,
        scale_bits: scale.to_bits(),
    }
}

fn accesskit_node(
    semantic: &SemanticNode,
    scale: f64,
    exposed: Option<&std::collections::HashSet<u64>>,
) -> Node {
    let role = match semantic.role {
        SemanticRole::Generic => Role::GenericContainer,
        SemanticRole::Group => Role::Group,
        SemanticRole::Label => Role::Label,
        SemanticRole::Heading => Role::Heading,
        SemanticRole::Button => Role::Button,
        SemanticRole::TextInput => Role::TextInput,
        SemanticRole::Image => Role::Image,
        SemanticRole::RadioGroup => Role::RadioGroup,
        SemanticRole::Region => Role::Region,
        SemanticRole::Link => Role::Link,
        SemanticRole::Dialog => Role::Dialog,
        SemanticRole::AlertDialog => Role::AlertDialog,
        SemanticRole::Alert => Role::Alert,
        SemanticRole::Status => Role::Status,
        SemanticRole::Tooltip => Role::Tooltip,
        SemanticRole::CheckBox => Role::CheckBox,
        SemanticRole::RadioButton => Role::RadioButton,
        SemanticRole::Switch => Role::Switch,
        SemanticRole::ComboBox => Role::ComboBox,
        SemanticRole::ListBox => Role::ListBox,
        SemanticRole::Option => Role::ListBoxOption,
        SemanticRole::Menu => Role::Menu,
        SemanticRole::MenuBar => Role::MenuBar,
        SemanticRole::MenuItem => Role::MenuItem,
        SemanticRole::Tree => Role::Tree,
        SemanticRole::TreeItem => Role::TreeItem,
        SemanticRole::Toolbar => Role::Toolbar,
        SemanticRole::Table => Role::Table,
        SemanticRole::Row => Role::Row,
        SemanticRole::Cell => Role::Cell,
        SemanticRole::ColumnHeader => Role::ColumnHeader,
        SemanticRole::RowHeader => Role::RowHeader,
        SemanticRole::Separator => Role::Splitter,
        SemanticRole::Slider => Role::Slider,
        SemanticRole::SpinButton => Role::SpinButton,
        SemanticRole::ProgressBar => Role::ProgressIndicator,
        SemanticRole::TabList => Role::TabList,
        SemanticRole::Tab => Role::Tab,
        SemanticRole::TabPanel => Role::TabPanel,
        SemanticRole::Grid => Role::Grid,
        SemanticRole::GridCell => Role::GridCell,
    };
    let mut node = Node::new(role);
    if let Some(label) = &semantic.label {
        node.set_label(label.clone());
    }
    if let Some(value) = &semantic.value {
        node.set_value(value.clone());
    }
    if let Some(value) = semantic.numeric_value {
        node.set_numeric_value(value);
    }
    if let Some(value) = semantic.min_numeric_value {
        node.set_min_numeric_value(value);
    }
    if let Some(value) = semantic.max_numeric_value {
        node.set_max_numeric_value(value);
    }
    let [x0, y0, x1, y1] = semantic.bounds.map(|value| f64::from(value) * scale);
    node.set_bounds(Rect::new(x0, y0, x1, y1));
    node.set_children(
        semantic
            .children
            .iter()
            .copied()
            .map(NodeId)
            .collect::<Vec<_>>(),
    );
    let relation_is_exposed = |id: &u64| exposed.is_none_or(|ids| ids.contains(id));
    node.set_controls(
        semantic
            .controls
            .iter()
            .filter(|id| relation_is_exposed(id))
            .copied()
            .map(NodeId)
            .collect::<Vec<_>>(),
    );
    if let Some(active) = semantic.active_descendant.filter(relation_is_exposed) {
        node.set_active_descendant(NodeId(active));
    }
    if semantic.disabled {
        node.set_disabled();
    }
    if let Some(state) = semantic.states.checked.or(semantic.states.pressed) {
        node.set_toggled(match state {
            SemanticToggleState::Off => Toggled::False,
            SemanticToggleState::On => Toggled::True,
            SemanticToggleState::Mixed => Toggled::Mixed,
        });
    }
    if let Some(selected) = semantic.states.selected {
        node.set_selected(selected);
    }
    if let Some(expanded) = semantic.states.expanded {
        node.set_expanded(expanded);
    }
    if let Some(current) = semantic.states.current {
        node.set_aria_current(match current {
            SemanticCurrent::True => AriaCurrent::True,
            SemanticCurrent::Page => AriaCurrent::Page,
            SemanticCurrent::Step => AriaCurrent::Step,
            SemanticCurrent::Location => AriaCurrent::Location,
            SemanticCurrent::Date => AriaCurrent::Date,
            SemanticCurrent::Time => AriaCurrent::Time,
        });
    }
    if let Some(popup) = semantic.states.popup {
        node.set_has_popup(match popup {
            SemanticPopup::Menu => HasPopup::Menu,
            SemanticPopup::ListBox => HasPopup::Listbox,
            SemanticPopup::Tree => HasPopup::Tree,
            SemanticPopup::Grid => HasPopup::Grid,
            SemanticPopup::Dialog => HasPopup::Dialog,
        });
    }
    if semantic.states.modal == Some(true) {
        node.set_modal();
    }
    node.add_action(Action::ScrollIntoView);
    match semantic.role {
        SemanticRole::Button
        | SemanticRole::Link
        | SemanticRole::CheckBox
        | SemanticRole::RadioButton
        | SemanticRole::Switch
        | SemanticRole::ComboBox
        | SemanticRole::Option
        | SemanticRole::MenuItem
        | SemanticRole::TreeItem
        | SemanticRole::Slider
        | SemanticRole::SpinButton
        | SemanticRole::Tab
            if !semantic.disabled =>
        {
            node.add_action(Action::Click);
            node.add_action(Action::Focus);
        }
        SemanticRole::TextInput | SemanticRole::ListBox if !semantic.disabled => {
            node.add_action(Action::Focus);
        }
        _ => {}
    }
    node
}

fn root_update(
    title: String,
    width: f64,
    height: f64,
    scale: f64,
    initialize: bool,
    snapshot: Option<&SemanticSnapshot>,
) -> TreeUpdate {
    let mut root = Node::new(Role::Window);
    root.set_label(title);
    root.set_bounds(Rect::new(0.0, 0.0, width, height));
    let children = snapshot
        .as_ref()
        .map(|snapshot| snapshot.exposed_root_children())
        .unwrap_or_default();
    root.set_children(children.iter().copied().map(NodeId).collect::<Vec<_>>());
    let mut tree = Tree::new(ROOT_ID);
    tree.toolkit_name = Some("Wabou".into());
    let focus = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.focus)
        .map(NodeId)
        .unwrap_or(ROOT_ID);
    let mut nodes = vec![(ROOT_ID, root)];
    if let Some(snapshot) = snapshot {
        let exposed = snapshot.exposed_nodes();
        let exposed_ids = exposed.iter().map(|node| node.id).collect();
        nodes.extend(exposed.into_iter().map(|semantic| {
            (
                NodeId(semantic.id),
                accesskit_node(semantic, scale, Some(&exposed_ids)),
            )
        }));
    }
    TreeUpdate {
        nodes,
        tree: initialize.then_some(tree),
        tree_id: TreeId::ROOT,
        focus,
    }
}

struct Handler {
    window: Arc<dyn Window>,
    events: Arc<Mutex<Vec<AccessKitEvent>>>,
}

impl EventHandler for Handler {
    fn handle_accesskit_event(&self, event: AccessKitEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
        self.window.request_redraw();
    }
}

/// Owns the platform adapter for Wabou's deliberately small semantic contract.
pub struct AccessibilityState {
    adapter: Adapter,
    events: Arc<Mutex<Vec<AccessKitEvent>>>,
    title: String,
    snapshot: Option<Arc<SemanticSnapshot>>,
    active: bool,
    initialize_pending: bool,
    last_published: Option<PublicationKey>,
}

impl AccessibilityState {
    /// Attach an AccessKit adapter to `window` without publishing a tree yet.
    pub fn new(window: Arc<dyn Window>, title: String) -> Self {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler = Arc::new(Handler {
            window: window.clone(),
            events: events.clone(),
        });
        let adapter = Adapter::with_combined_handler(
            window
                .window_handle()
                .expect("winit window must expose a raw window handle")
                .as_raw(),
            handler,
        );
        Self {
            adapter,
            events,
            title,
            snapshot: None,
            active: false,
            initialize_pending: false,
            last_published: None,
        }
    }

    /// Forward window focus, movement, and resize changes to AccessKit.
    pub fn process_window_event(&mut self, window: &dyn Window, event: &WindowEvent) {
        match event {
            WindowEvent::Focused(focused) => self.adapter.set_focus(*focused),
            WindowEvent::Moved(_) | WindowEvent::SurfaceResized(_) => {
                self.update_window_bounds(window)
            }
            _ => {}
        }
    }

    /// Replace the immutable semantic projection used by the next publication.
    pub fn set_snapshot(&mut self, snapshot: Option<Arc<SemanticSnapshot>>) {
        self.snapshot = snapshot;
    }

    /// Consume activation lifecycle events before a frame is built and report
    /// whether the frame source should produce semantic data.
    pub fn prepare_frame(&mut self) -> bool {
        if let Ok(mut events) = self.events.lock() {
            events.retain(|event| match event {
                AccessKitEvent::InitialTreeRequested => {
                    self.active = true;
                    self.initialize_pending = true;
                    false
                }
                AccessKitEvent::AccessibilityDeactivated => {
                    self.active = false;
                    self.initialize_pending = false;
                    self.snapshot = None;
                    self.last_published = None;
                    false
                }
                AccessKitEvent::ActionRequested(_) => true,
            });
        }
        self.active
    }

    /// Drain supported platform actions for routing back to the frame source.
    pub fn take_actions(&mut self) -> Vec<SemanticAction> {
        self.events
            .lock()
            .map(|mut events| {
                let mut actions = Vec::new();
                events.retain(|event| {
                    let AccessKitEvent::ActionRequested(request) = event else {
                        return true;
                    };
                    let target = request.target_node.0;
                    let action = match request.action {
                        Action::Click => Some(SemanticAction::Click { target }),
                        Action::Focus => Some(SemanticAction::Focus { target }),
                        Action::Blur => Some(SemanticAction::Blur { target }),
                        Action::ScrollIntoView => Some(SemanticAction::ScrollIntoView { target }),
                        _ => None,
                    };
                    actions.extend(action);
                    false
                });
                actions
            })
            .unwrap_or_default()
    }

    /// Publish the current semantic tree when activation or its revision requires it.
    pub fn publish_root(&mut self, window: &dyn Window) {
        self.update_window_bounds(window);
        if !self.active {
            return;
        }
        let requested = std::mem::take(&mut self.initialize_pending);
        let title = self.title.clone();
        let snapshot = self.snapshot.clone();
        let size = window.surface_size().cast::<f64>();
        let scale = window.scale_factor();
        let key = publication_key(
            snapshot.as_deref(),
            size.width as u32,
            size.height as u32,
            scale,
        );
        if !requested && self.last_published == Some(key) {
            return;
        }
        self.last_published = Some(key);
        self.adapter.update_if_active(move || {
            root_update(
                title,
                size.width,
                size.height,
                scale,
                requested,
                snapshot.as_deref(),
            )
        });
    }

    fn update_window_bounds(&mut self, window: &dyn Window) {
        let outer_origin: (_, _) = window
            .outer_position()
            .unwrap_or_default()
            .cast::<f64>()
            .into();
        let outer_size: (_, _) = window.outer_size().cast::<f64>().into();
        let inner_origin: (_, _) = window.surface_position().cast::<f64>().into();
        let inner_size: (_, _) = window.surface_size().cast::<f64>().into();
        self.adapter.set_window_bounds(
            Rect::from_origin_size(outer_origin, outer_size),
            Rect::from_origin_size(inner_origin, inner_size),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_tree_identifies_the_wabou_window() {
        let update = root_update("Gallery".into(), 800.0, 600.0, 1.0, true, None);
        assert_eq!(update.focus, ROOT_ID);
        assert_eq!(update.tree.as_ref().unwrap().root, ROOT_ID);
        assert_eq!(
            update.tree.as_ref().unwrap().toolkit_name.as_deref(),
            Some("Wabou")
        );
        assert_eq!(update.nodes[0].1.role(), Role::Window);
        assert_eq!(update.nodes[0].1.label(), Some("Gallery"));
        assert_eq!(
            update.nodes[0].1.bounds(),
            Some(Rect::new(0.0, 0.0, 800.0, 600.0))
        );
    }

    #[test]
    fn live_region_roles_reach_accesskit() {
        let snapshot = SemanticSnapshot {
            revision: 1,
            nodes: vec![
                crate::SemanticNode {
                    id: 2,
                    role: SemanticRole::Status,
                    label: Some("Saved".into()),
                    value: Some("complete".into()),
                    numeric_value: None,
                    min_numeric_value: None,
                    max_numeric_value: None,
                    bounds: [0.0, 0.0, 100.0, 20.0],
                    children: vec![],
                    controls: vec![],
                    active_descendant: None,
                    disabled: false,
                    states: crate::SemanticStates::default(),
                },
                crate::SemanticNode {
                    id: 3,
                    role: SemanticRole::Alert,
                    label: Some("Connection lost".into()),
                    value: None,
                    numeric_value: None,
                    min_numeric_value: None,
                    max_numeric_value: None,
                    bounds: [0.0, 20.0, 100.0, 40.0],
                    children: vec![],
                    controls: vec![],
                    active_descendant: None,
                    disabled: false,
                    states: crate::SemanticStates::default(),
                },
            ],
            root_children: vec![2, 3],
            ..SemanticSnapshot::default()
        };
        let update = root_update("Gallery".into(), 200.0, 100.0, 1.0, true, Some(&snapshot));
        assert_eq!(update.nodes[1].1.role(), Role::Status);
        assert_eq!(update.nodes[1].1.label(), Some("Saved"));
        assert_eq!(update.nodes[1].1.value(), Some("complete"));
        assert_eq!(update.nodes[2].1.role(), Role::Alert);
        assert_eq!(update.nodes[2].1.label(), Some("Connection lost"));
    }

    #[test]
    fn interaction_states_reach_accesskit() {
        let semantic = crate::SemanticNode {
            id: 2,
            role: SemanticRole::CheckBox,
            label: Some("Partial selection".into()),
            value: None,
            numeric_value: None,
            min_numeric_value: None,
            max_numeric_value: None,
            bounds: [0.0, 0.0, 100.0, 20.0],
            children: vec![],
            controls: vec![],
            active_descendant: None,
            disabled: false,
            states: crate::SemanticStates {
                checked: Some(SemanticToggleState::Mixed),
                selected: Some(true),
                expanded: Some(false),
                current: Some(SemanticCurrent::Date),
                popup: Some(SemanticPopup::ListBox),
                modal: Some(true),
                ..crate::SemanticStates::default()
            },
        };

        let node = accesskit_node(&semantic, 1.0, None);
        assert_eq!(node.toggled(), Some(Toggled::Mixed));
        assert_eq!(node.is_selected(), Some(true));
        assert_eq!(node.is_expanded(), Some(false));
        assert_eq!(node.aria_current(), Some(AriaCurrent::Date));
        assert_eq!(node.has_popup(), Some(HasPopup::Listbox));
        assert!(node.is_modal());
    }

    #[test]
    fn numeric_range_reaches_accesskit() {
        let semantic = crate::SemanticNode {
            id: 2,
            role: SemanticRole::ProgressBar,
            label: Some("Build progress".into()),
            value: Some("64 percent".into()),
            numeric_value: Some(64.0),
            min_numeric_value: Some(0.0),
            max_numeric_value: Some(100.0),
            bounds: [0.0, 0.0, 100.0, 8.0],
            children: vec![],
            controls: vec![],
            active_descendant: None,
            disabled: false,
            states: crate::SemanticStates::default(),
        };

        let node = accesskit_node(&semantic, 1.0, None);
        assert_eq!(node.role(), Role::ProgressIndicator);
        assert_eq!(node.value(), Some("64 percent"));
        assert_eq!(node.numeric_value(), Some(64.0));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(100.0));
    }

    #[test]
    fn composite_widget_relations_reach_accesskit_and_filter_hidden_targets() {
        let semantic = crate::SemanticNode {
            id: 2,
            role: SemanticRole::ComboBox,
            label: Some("Workspace".into()),
            value: None,
            numeric_value: None,
            min_numeric_value: None,
            max_numeric_value: None,
            bounds: [0.0, 0.0, 100.0, 20.0],
            children: vec![],
            controls: vec![3, 9],
            active_descendant: Some(4),
            disabled: false,
            states: crate::SemanticStates::default(),
        };

        let node = accesskit_node(&semantic, 1.0, None);
        assert_eq!(node.controls(), &[NodeId(3), NodeId(9)]);
        assert_eq!(node.active_descendant(), Some(NodeId(4)));

        let exposed = [2, 3].into_iter().collect();
        let filtered = accesskit_node(&semantic, 1.0, Some(&exposed));
        assert_eq!(filtered.controls(), &[NodeId(3)]);
        assert_eq!(filtered.active_descendant(), None);
    }

    #[test]
    fn structural_roles_reach_accesskit_without_becoming_generic_containers() {
        let roles = [
            (SemanticRole::Group, Role::Group),
            (SemanticRole::Dialog, Role::Dialog),
            (SemanticRole::AlertDialog, Role::AlertDialog),
            (SemanticRole::Image, Role::Image),
            (SemanticRole::RadioGroup, Role::RadioGroup),
            (SemanticRole::Region, Role::Region),
            (SemanticRole::Menu, Role::Menu),
            (SemanticRole::MenuBar, Role::MenuBar),
            (SemanticRole::MenuItem, Role::MenuItem),
            (SemanticRole::Tree, Role::Tree),
            (SemanticRole::TreeItem, Role::TreeItem),
            (SemanticRole::Toolbar, Role::Toolbar),
            (SemanticRole::Table, Role::Table),
            (SemanticRole::Row, Role::Row),
            (SemanticRole::Cell, Role::Cell),
            (SemanticRole::ColumnHeader, Role::ColumnHeader),
            (SemanticRole::RowHeader, Role::RowHeader),
            (SemanticRole::Separator, Role::Splitter),
            (SemanticRole::Slider, Role::Slider),
            (SemanticRole::SpinButton, Role::SpinButton),
            (SemanticRole::ProgressBar, Role::ProgressIndicator),
            (SemanticRole::Heading, Role::Heading),
            (SemanticRole::TabList, Role::TabList),
            (SemanticRole::Tab, Role::Tab),
            (SemanticRole::TabPanel, Role::TabPanel),
            (SemanticRole::Grid, Role::Grid),
            (SemanticRole::GridCell, Role::GridCell),
        ];
        for (index, (semantic, _)) in roles.iter().enumerate() {
            let node = crate::SemanticNode {
                id: index as u64 + 2,
                role: *semantic,
                label: Some(format!("table node {index}")),
                value: None,
                numeric_value: None,
                min_numeric_value: None,
                max_numeric_value: None,
                bounds: [0.0, index as f32 * 20.0, 100.0, index as f32 * 20.0 + 20.0],
                children: vec![],
                controls: vec![],
                active_descendant: None,
                disabled: false,
                states: crate::SemanticStates::default(),
            };
            assert_eq!(accesskit_node(&node, 1.0, None).role(), roles[index].1);
        }
    }

    #[test]
    fn modal_snapshot_exposes_only_the_modal_at_the_window_root() {
        let snapshot = SemanticSnapshot {
            revision: 1,
            nodes: vec![
                crate::SemanticNode {
                    id: 2,
                    role: SemanticRole::Button,
                    label: Some("Background".into()),
                    value: None,
                    numeric_value: None,
                    min_numeric_value: None,
                    max_numeric_value: None,
                    bounds: [0.0, 0.0, 50.0, 20.0],
                    children: vec![],
                    controls: vec![],
                    active_descendant: None,
                    disabled: false,
                    states: crate::SemanticStates::default(),
                },
                crate::SemanticNode {
                    id: 3,
                    role: SemanticRole::Dialog,
                    label: Some("Settings".into()),
                    value: None,
                    numeric_value: None,
                    min_numeric_value: None,
                    max_numeric_value: None,
                    bounds: [10.0, 10.0, 90.0, 90.0],
                    children: vec![4],
                    controls: vec![],
                    active_descendant: None,
                    disabled: false,
                    states: crate::SemanticStates::default(),
                },
                crate::SemanticNode {
                    id: 4,
                    role: SemanticRole::Button,
                    label: Some("Save".into()),
                    value: None,
                    numeric_value: None,
                    min_numeric_value: None,
                    max_numeric_value: None,
                    bounds: [60.0, 60.0, 80.0, 75.0],
                    children: vec![],
                    controls: vec![],
                    active_descendant: None,
                    disabled: false,
                    states: crate::SemanticStates::default(),
                },
            ],
            root_children: vec![2, 3],
            focus: Some(3),
            modal_root: Some(3),
        };
        let update = root_update("Gallery".into(), 200.0, 200.0, 2.0, true, Some(&snapshot));
        assert_eq!(update.focus, NodeId(3));
        assert_eq!(update.nodes[0].1.children(), &[NodeId(3)]);
        assert_eq!(update.nodes.len(), 3, "background subtree must be absent");
        assert_eq!(update.nodes[1].1.role(), Role::Dialog);
        assert_eq!(update.nodes[1].1.label(), Some("Settings"));
        assert_eq!(
            update.nodes[1].1.bounds(),
            Some(Rect::new(20.0, 20.0, 180.0, 180.0))
        );
        assert!(update.nodes[2].1.supports_action(Action::Click));
        assert!(update.nodes[2].1.supports_action(Action::Focus));
        assert!(update.nodes[2].1.supports_action(Action::ScrollIntoView));
    }

    #[test]
    fn publication_revision_changes_only_for_semantics_or_window_geometry() {
        let snapshot = SemanticSnapshot {
            revision: 7,
            ..SemanticSnapshot::default()
        };
        let initial = publication_key(Some(&snapshot), 800, 600, 2.0);
        assert_eq!(initial, publication_key(Some(&snapshot), 800, 600, 2.0));

        let mut changed = snapshot;
        changed.revision += 1;
        assert_ne!(initial, publication_key(Some(&changed), 800, 600, 2.0));
        assert_ne!(initial, publication_key(Some(&changed), 801, 600, 2.0));
    }
}
