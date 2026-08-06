//! Accessibility data exchanged between a frame source and the native host.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticRole {
    Generic,
    Label,
    Button,
    TextInput,
    Image,
    Link,
    Dialog,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticNode {
    pub id: u64,
    pub role: SemanticRole,
    pub label: Option<String>,
    pub bounds: [f32; 4],
    pub children: Vec<u64>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemanticSnapshot {
    pub nodes: Vec<SemanticNode>,
    pub root_children: Vec<u64>,
    pub focus: Option<u64>,
    /// When set, only this modal subtree is exposed below the window root.
    pub modal_root: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticAction {
    Click { target: u64 },
    Focus { target: u64 },
    Blur { target: u64 },
}
