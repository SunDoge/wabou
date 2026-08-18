//! Accessibility data exchanged between a frame source and the native host.

use std::collections::{HashMap, HashSet};

macro_rules! semantic_roles {
    ($( $(#[$doc:meta])* $variant:ident => $name:literal ),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        /// Renderer-independent role understood by Wabou's accessibility bridge.
        pub enum SemanticRole {
            $( $(#[$doc])* $variant, )+
        }

        impl SemanticRole {
            /// Every role supported by the native semantic contract.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Parse the canonical role name used by the JS semantic contract.
            pub fn from_name(name: &str) -> Option<Self> {
                // Generic is an internal fallback, not an authored semantic role.
                if name == "generic" {
                    return None;
                }
                match name {
                    $( $name => Some(Self::$variant), )+
                    _ => None,
                }
            }

            /// Canonical role name exposed to JS tests and diagnostics.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $name, )+
                }
            }
        }
    };
}

semantic_roles! {
    /// Container without a more specific semantic role.
    Generic => "generic",
    /// Named or structural group of related controls.
    Group => "group",
    /// Static text label.
    Label => "label",
    /// Section or page heading.
    Heading => "heading",
    /// Activatable button.
    Button => "button",
    /// Editable text field.
    TextInput => "textbox",
    /// Informative image.
    Image => "img",
    /// Container for mutually exclusive radio controls.
    RadioGroup => "radiogroup",
    /// Navigational link.
    Link => "link",
    /// Dialog surface.
    Dialog => "dialog",
    /// Assertive alert message.
    Alert => "alert",
    /// Non-assertive status message.
    Status => "status",
    /// Boolean checkbox.
    CheckBox => "checkbox",
    /// Mutually exclusive radio item.
    RadioButton => "radio",
    /// On/off switch.
    Switch => "switch",
    /// Control that opens a list of choices.
    ComboBox => "combobox",
    /// List of selectable options.
    ListBox => "listbox",
    /// Selectable item in a list box.
    Option => "option",
    /// Container for application or context commands.
    Menu => "menu",
    /// One activatable command in a menu.
    MenuItem => "menuitem",
    /// Hierarchical collection of expandable items.
    Tree => "tree",
    /// One item in a hierarchical tree.
    TreeItem => "treeitem",
    /// Tabular data container.
    Table => "table",
    /// One row in a table.
    Row => "row",
    /// One data cell in a table row.
    Cell => "cell",
    /// Header describing a table column.
    ColumnHeader => "columnheader",
    /// Header describing a table row.
    RowHeader => "rowheader",
    /// Numeric value selected along a bounded range.
    Slider => "slider",
    /// Read-only completion value along a bounded range.
    ProgressBar => "progressbar",
    /// Container for a set of tabs.
    TabList => "tablist",
    /// One selectable tab.
    Tab => "tab",
    /// Content controlled by a tab.
    TabPanel => "tabpanel",
    /// Interactive tabular grid.
    Grid => "grid",
    /// One cell in an interactive grid.
    GridCell => "gridcell",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Meaning of the item currently active within a related set.
pub enum SemanticCurrent {
    /// Current item without a more specific category.
    True,
    /// Current page.
    Page,
    /// Current step in a process.
    Step,
    /// Current location.
    Location,
    /// Current date.
    Date,
    /// Current time.
    Time,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Kind of popup opened by a semantic control.
pub enum SemanticPopup {
    /// Command menu.
    Menu,
    /// List of selectable options.
    ListBox,
    /// Hierarchical tree.
    Tree,
    /// Interactive grid.
    Grid,
    /// Dialog surface.
    Dialog,
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
    /// Current item state for navigation, progress, dates, and times.
    pub current: Option<SemanticCurrent>,
    /// Kind of popup opened by this control.
    pub popup: Option<SemanticPopup>,
    /// Whether this node is an explicitly modal surface.
    pub modal: Option<bool>,
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
    /// Current numeric value for range-based controls.
    pub numeric_value: Option<f64>,
    /// Lower bound for a numeric value.
    pub min_numeric_value: Option<f64>,
    /// Upper bound for a numeric value.
    pub max_numeric_value: Option<f64>,
    /// Bounds `(x0, y0, x1, y1)` in logical window coordinates.
    pub bounds: [f32; 4],
    /// Ordered semantic child identifiers.
    pub children: Vec<u64>,
    /// Semantic nodes controlled by this node through `aria-controls`.
    pub controls: Vec<u64>,
    /// Current composite-widget item referenced by `aria-activedescendant`.
    pub active_descendant: Option<u64>,
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

impl SemanticSnapshot {
    /// Return the node ids attached directly below the platform accessibility
    /// root after modal isolation is applied.
    pub fn exposed_root_children(&self) -> Vec<u64> {
        self.modal_root
            .map_or_else(|| self.root_children.clone(), |modal| vec![modal])
    }

    /// Return nodes reachable from the platform accessibility root in logical
    /// source order. While a modal is active, background nodes remain in the
    /// immutable snapshot for diagnostics but are not exposed here.
    pub fn exposed_nodes(&self) -> Vec<&SemanticNode> {
        let by_id = self
            .nodes
            .iter()
            .map(|node| (node.id, node))
            .collect::<HashMap<_, _>>();
        let mut stack = self
            .exposed_root_children()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();
        let mut exposed = HashSet::new();
        while let Some(id) = stack.pop() {
            if !exposed.insert(id) {
                continue;
            }
            if let Some(node) = by_id.get(&id) {
                stack.extend(node.children.iter().rev().copied());
            }
        }
        self.nodes
            .iter()
            .filter(|node| exposed.contains(&node.id))
            .collect()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: u64, children: &[u64]) -> SemanticNode {
        SemanticNode {
            id,
            role: SemanticRole::Generic,
            label: None,
            value: None,
            numeric_value: None,
            min_numeric_value: None,
            max_numeric_value: None,
            bounds: [0.0; 4],
            children: children.to_vec(),
            controls: Vec::new(),
            active_descendant: None,
            disabled: false,
            states: SemanticStates::default(),
        }
    }

    #[test]
    fn semantic_role_names_round_trip_through_one_contract() {
        for role in SemanticRole::ALL
            .iter()
            .filter(|role| **role != SemanticRole::Generic)
        {
            assert_eq!(SemanticRole::from_name(role.as_str()), Some(*role));
        }
        assert_eq!(SemanticRole::from_name("generic"), None);
        assert_eq!(SemanticRole::from_name("alertdialog"), None);
    }

    #[test]
    fn exposed_nodes_follow_reachable_source_order_and_isolate_a_modal() {
        let mut snapshot = SemanticSnapshot {
            nodes: vec![node(1, &[]), node(2, &[3]), node(3, &[]), node(4, &[])],
            root_children: vec![1, 2],
            ..SemanticSnapshot::default()
        };
        assert_eq!(snapshot.exposed_root_children(), [1, 2]);
        assert_eq!(
            snapshot
                .exposed_nodes()
                .into_iter()
                .map(|node| node.id)
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );

        snapshot.modal_root = Some(2);
        assert_eq!(snapshot.exposed_root_children(), [2]);
        assert_eq!(
            snapshot
                .exposed_nodes()
                .into_iter()
                .map(|node| node.id)
                .collect::<Vec<_>>(),
            [2, 3]
        );
    }
}
