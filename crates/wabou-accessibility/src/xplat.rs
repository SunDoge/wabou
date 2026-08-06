//! Window-level AccessKit adapter and retained accessibility tree publication.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use accesskit::{Action, Node, NodeId, Rect, Role, Tree, TreeId, TreeUpdate};
use accesskit_xplat::{Adapter, EventHandler, WindowEvent as AccessKitEvent};
use raw_window_handle::HasWindowHandle;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::{SemanticAction, SemanticRole, SemanticSnapshot};

const ROOT_ID: NodeId = NodeId(0);

fn root_update(
    title: String,
    width: f64,
    height: f64,
    scale: f64,
    initialize: bool,
    snapshot: Option<SemanticSnapshot>,
) -> TreeUpdate {
    let mut root = Node::new(Role::Window);
    root.set_label(title);
    root.set_bounds(Rect::new(0.0, 0.0, width, height));
    let children = snapshot
        .as_ref()
        .map(|snapshot| {
            snapshot
                .modal_root
                .map_or_else(|| snapshot.root_children.clone(), |modal| vec![modal])
        })
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
        let allowed = snapshot.modal_root.map(|modal| {
            let by_id: HashMap<_, _> = snapshot.nodes.iter().map(|node| (node.id, node)).collect();
            let mut allowed = HashSet::new();
            let mut pending = vec![modal];
            while let Some(id) = pending.pop() {
                if allowed.insert(id)
                    && let Some(node) = by_id.get(&id)
                {
                    pending.extend(node.children.iter().copied());
                }
            }
            allowed
        });
        nodes.extend(
            snapshot
                .nodes
                .into_iter()
                .filter(|semantic| {
                    allowed
                        .as_ref()
                        .is_none_or(|allowed| allowed.contains(&semantic.id))
                })
                .map(|semantic| {
                    let role = match semantic.role {
                        SemanticRole::Generic => Role::GenericContainer,
                        SemanticRole::Label => Role::Label,
                        SemanticRole::Button => Role::Button,
                        SemanticRole::TextInput => Role::TextInput,
                        SemanticRole::Image => Role::Image,
                        SemanticRole::Link => Role::Link,
                        SemanticRole::Dialog => Role::Dialog,
                    };
                    let mut node = Node::new(role);
                    if let Some(label) = semantic.label {
                        node.set_label(label);
                    }
                    let [x0, y0, x1, y1] = semantic.bounds.map(|value| f64::from(value) * scale);
                    node.set_bounds(Rect::new(x0, y0, x1, y1));
                    node.set_children(
                        semantic
                            .children
                            .into_iter()
                            .map(NodeId)
                            .collect::<Vec<_>>(),
                    );
                    if semantic.disabled {
                        node.set_disabled();
                    }
                    match semantic.role {
                        SemanticRole::Button | SemanticRole::Link if !semantic.disabled => {
                            node.add_action(Action::Click);
                            node.add_action(Action::Focus);
                        }
                        SemanticRole::TextInput if !semantic.disabled => {
                            node.add_action(Action::Focus);
                        }
                        _ => {}
                    }
                    (NodeId(semantic.id), node)
                }),
        );
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
    snapshot: Option<SemanticSnapshot>,
}

impl AccessibilityState {
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
        }
    }

    pub fn process_window_event(&mut self, window: &dyn Window, event: &WindowEvent) {
        match event {
            WindowEvent::Focused(focused) => self.adapter.set_focus(*focused),
            WindowEvent::Moved(_) | WindowEvent::SurfaceResized(_) => {
                self.update_window_bounds(window)
            }
            _ => {}
        }
    }

    pub fn set_snapshot(&mut self, snapshot: Option<SemanticSnapshot>) {
        self.snapshot = snapshot;
    }

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
                        _ => None,
                    };
                    actions.extend(action);
                    false
                });
                actions
            })
            .unwrap_or_default()
    }

    pub fn publish_root(&mut self, window: &dyn Window) {
        self.update_window_bounds(window);
        let requested = self
            .events
            .lock()
            .map(|mut events| {
                let requested = events
                    .iter()
                    .any(|event| matches!(event, AccessKitEvent::InitialTreeRequested));
                events.clear();
                requested
            })
            .unwrap_or(false);
        let title = self.title.clone();
        let snapshot = self.snapshot.clone();
        let size = window.surface_size().cast::<f64>();
        let scale = window.scale_factor();
        self.adapter.update_if_active(move || {
            root_update(title, size.width, size.height, scale, requested, snapshot)
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
    fn modal_snapshot_exposes_only_the_modal_at_the_window_root() {
        let snapshot = SemanticSnapshot {
            nodes: vec![
                crate::SemanticNode {
                    id: 2,
                    role: SemanticRole::Button,
                    label: Some("Background".into()),
                    bounds: [0.0, 0.0, 50.0, 20.0],
                    children: vec![],
                    disabled: false,
                },
                crate::SemanticNode {
                    id: 3,
                    role: SemanticRole::Dialog,
                    label: Some("Settings".into()),
                    bounds: [10.0, 10.0, 90.0, 90.0],
                    children: vec![4],
                    disabled: false,
                },
                crate::SemanticNode {
                    id: 4,
                    role: SemanticRole::Button,
                    label: Some("Save".into()),
                    bounds: [60.0, 60.0, 80.0, 75.0],
                    children: vec![],
                    disabled: false,
                },
            ],
            root_children: vec![2, 3],
            focus: Some(3),
            modal_root: Some(3),
        };
        let update = root_update("Gallery".into(), 200.0, 200.0, 2.0, true, Some(snapshot));
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
    }
}
