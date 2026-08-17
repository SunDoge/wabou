//! Accessibility data exchanged between a frame source and the native host.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Renderer-independent role understood by Wabou's accessibility bridge.
pub enum SemanticRole {
    /// Container without a more specific semantic role.
    Generic,
    /// Static text label.
    Label,
    /// Activatable button.
    Button,
    /// Editable text field.
    TextInput,
    /// Informative image.
    Image,
    /// Navigational link.
    Link,
    /// Dialog surface.
    Dialog,
    /// Assertive alert message.
    Alert,
    /// Non-assertive status message.
    Status,
    /// Boolean checkbox.
    CheckBox,
    /// Mutually exclusive radio item.
    RadioButton,
    /// On/off switch.
    Switch,
    /// Control that opens a list of choices.
    ComboBox,
    /// List of selectable options.
    ListBox,
    /// Selectable item in a list box.
    Option,
    /// Tabular data container.
    Table,
    /// One row in a table.
    Row,
    /// One data cell in a table row.
    Cell,
    /// Header describing a table column.
    ColumnHeader,
    /// Header describing a table row.
    RowHeader,
    /// Numeric value selected along a bounded range.
    Slider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State of a checkable or pressable semantic control.
pub enum SemanticToggleState {
    /// The control is unchecked or unpressed.
    Off,
    /// The control is checked or pressed.
    On,
    /// A checkbox represents a mixture of checked and unchecked values.
    Mixed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Optional interaction states attached to one semantic node.
pub struct SemanticStates {
    /// Checkbox, radio, or switch state when applicable.
    pub checked: Option<SemanticToggleState>,
    /// Toggle-button state when applicable.
    pub pressed: Option<SemanticToggleState>,
    /// Selection state for options, tabs, rows, and similar items.
    pub selected: Option<bool>,
    /// Expansion state for disclosure controls and combo boxes.
    pub expanded: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
/// One node in a window's platform-neutral semantic tree.
pub struct SemanticNode {
    /// Stable identifier within the snapshot's window.
    pub id: u64,
    /// Platform-neutral role.
    pub role: SemanticRole,
    /// Accessible name.
    pub label: Option<String>,
    /// Current textual value for value-bearing controls.
    pub value: Option<String>,
    /// Bounds `(x0, y0, x1, y1)` in logical window coordinates.
    pub bounds: [f32; 4],
    /// Ordered semantic child identifiers.
    pub children: Vec<u64>,
    /// Whether assistive technology should expose the node as disabled.
    pub disabled: bool,
    /// Interaction states that are meaningful for this node.
    pub states: SemanticStates,
}

#[derive(Debug, Clone, Default, PartialEq)]
/// Complete immutable semantic projection for one window.
pub struct SemanticSnapshot {
    /// Monotonic source revision; unchanged revisions need not be republished.
    pub revision: u64,
    /// Semantic nodes in source order.
    pub nodes: Vec<SemanticNode>,
    /// Children attached directly below the platform window root.
    pub root_children: Vec<u64>,
    /// Focused semantic node, if any.
    pub focus: Option<u64>,
    /// When set, only this modal subtree is exposed below the window root.
    pub modal_root: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Action requested by assistive technology and routed back to the UI source.
pub enum SemanticAction {
    /// Activate a node through the same route as a pointer click.
    Click {
        /// Target semantic node identifier.
        target: u64,
    },
    /// Move semantic and keyboard focus to a node.
    Focus {
        /// Target semantic node identifier.
        target: u64,
    },
    /// Clear focus from a node.
    Blur {
        /// Target semantic node identifier.
        target: u64,
    },
    /// Reveal a node inside its nearest scroll containers.
    ScrollIntoView {
        /// Target semantic node identifier.
        target: u64,
    },
}
