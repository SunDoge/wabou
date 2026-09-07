//! AccessKit publication adapter for Wabou's Winit/Vello backend.

#![warn(missing_docs)]

pub use wabou_accessibility::*;

mod xplat;

pub use xplat::AccessibilityState;
