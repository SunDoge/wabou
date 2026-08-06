//! Window-level AccessKit adapter and retained accessibility tree publication.

use std::sync::{Arc, Mutex};

use accesskit::{Node, NodeId, Rect, Role, Tree, TreeId, TreeUpdate};
use accesskit_xplat::{Adapter, EventHandler, WindowEvent as AccessKitEvent};
use raw_window_handle::HasWindowHandle;
use winit::event::WindowEvent;
use winit::window::Window;

const ROOT_ID: NodeId = NodeId(0);

fn root_update(title: String, width: f64, height: f64, initialize: bool) -> TreeUpdate {
    let mut root = Node::new(Role::Window);
    root.set_label(title);
    root.set_bounds(Rect::new(0.0, 0.0, width, height));
    let mut tree = Tree::new(ROOT_ID);
    tree.toolkit_name = Some("Wabou".into());
    TreeUpdate {
        nodes: vec![(ROOT_ID, root)],
        tree: initialize.then_some(tree),
        tree_id: TreeId::ROOT,
        focus: ROOT_ID,
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

/// Owns the platform adapter. Semantic descendants will be added by the
/// retained Wabou tree; the window root is already exposed in this first stage.
pub struct AccessibilityState {
    adapter: Adapter,
    events: Arc<Mutex<Vec<AccessKitEvent>>>,
    title: String,
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
        let size = window.surface_size().cast::<f64>();
        self.adapter
            .update_if_active(move || root_update(title, size.width, size.height, requested));
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
        let update = root_update("Gallery".into(), 800.0, 600.0, true);
        assert_eq!(update.focus, ROOT_ID);
        assert_eq!(update.tree.as_ref().unwrap().root, ROOT_ID);
        assert_eq!(update.tree.as_ref().unwrap().toolkit_name.as_deref(), Some("Wabou"));
        assert_eq!(update.nodes[0].1.role(), Role::Window);
        assert_eq!(update.nodes[0].1.label(), Some("Gallery"));
        assert_eq!(
            update.nodes[0].1.bounds(),
            Some(Rect::new(0.0, 0.0, 800.0, 600.0))
        );
    }
}
