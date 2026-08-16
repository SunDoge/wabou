//! Native utility-class parser for Wabou.
//!
//! The Rust parser is authoritative. [`manifest`] exports the same scales,
//! colors and dynamic rule families for editor/build tooling.

#![warn(missing_docs)]

mod manifest;
mod model;
mod rules;
mod theme;

pub use manifest::{MANIFEST_VERSION, manifest, manifest_with_theme};
pub use model::{
    Color, Declaration, DynamicPrefix, DynamicRule, Length, Manifest, ParsedUtility, Theme, Value,
};
pub use rules::{ParseError, parse_utility, parse_utility_with_theme};
