//! Data protocol for one native library exporting multiple implementations.
use serde::{Deserialize, Serialize};
/// Registry protocol revision in the existing ModuleApi contract field.
pub const PLUGIN_CONTRACT_VERSION: u32 = 0x5750_4c31;
/// Reserved synchronous query, called before publishing a candidate library.
pub const MANIFEST_METHOD: &str = "__wabou_plugin_manifest_v1";
/// A registry entry's namespace. Capability and widget names may overlap.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportKind {
    /// QuickJS-visible host module.
    Capability,
    /// Native widget callbacks.
    Widget,
}
/// Metadata copied into the host before replacing any live implementation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    /// Implementation namespace.
    pub kind: ExportKind,
    /// Stable exported capability name or widget tag.
    pub name: String,
    /// Application contract/state schema revision.
    pub version: u32,
}
