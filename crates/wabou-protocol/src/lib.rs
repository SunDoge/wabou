//! Backend-neutral binary UI protocol shared by Wabou runtimes.

#![warn(missing_docs)]

mod atom;
mod protocol;
mod wire;

pub use atom::{Atom, AtomPool};
pub use protocol::*;
pub use wire::*;
