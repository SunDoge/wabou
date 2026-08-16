//! Renderer-independent accessibility semantics and platform publication.
//!
//! This crate intentionally proves a small end-to-end contract: publish basic
//! roles, labels, bounds and focus, then route click/focus actions back to the
//! frame source. It is not yet a complete accessibility implementation.

#![warn(missing_docs)]

mod model;
mod xplat;

pub use model::{SemanticAction, SemanticNode, SemanticRole, SemanticSnapshot};
pub use xplat::AccessibilityState;
