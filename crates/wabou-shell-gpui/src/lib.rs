//! GPUI-CE platform shell for Wabou.
//!
//! Solid owns application reactivity. This crate owns the native window and
//! projects completed Solid flushes into GPUI. The projection boundary is
//! deliberately frame-oriented: individual property writes never notify GPUI.

mod element;
mod projection;
mod style;
mod tree;

pub use element::ProjectedElement;
pub use gpui;
pub use projection::{DirtyKind, FrameBatch, NodeKey, PendingNode};
pub use style::{StyleDiagnostic, StyleProjection};
pub use tree::{ProjectedNode, ProjectionError, ProjectionTree};

/// Run a GPUI application using Wabou's selected platform implementation.
///
/// Keeping application construction here prevents downstream applications from
/// depending directly on `gpui_ce_platform` and gives Wabou one place to install
/// platform services as the migration progresses.
pub fn application() -> gpui::Application {
    gpui_platform::application()
}
