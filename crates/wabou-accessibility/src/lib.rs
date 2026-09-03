//! Renderer-independent accessibility semantics.
//!
//! This crate intentionally proves a small end-to-end contract: publish basic
//! roles, labels, bounds, focus, and common interaction states, then route
//! click/focus actions back to the frame source. It is not yet a complete
//! accessibility implementation.

#![warn(missing_docs)]

mod model;

pub use model::{
    SemanticAction, SemanticCurrent, SemanticNode, SemanticPopup, SemanticRole, SemanticSnapshot,
    SemanticStates, SemanticToggleState,
};
