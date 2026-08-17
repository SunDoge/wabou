//! Renderer-independent accessibility semantics and platform publication.
//!
//! This crate intentionally proves a small end-to-end contract: publish basic
//! roles, labels, bounds, focus, and common interaction states, then route
//! click/focus actions back to the frame source. It is not yet a complete
//! accessibility implementation.

#![warn(missing_docs)]

mod model;
mod xplat;

pub use model::{
    SemanticAction, SemanticNode, SemanticRole, SemanticSnapshot, SemanticStates,
    SemanticToggleState,
};
pub use xplat::AccessibilityState;
