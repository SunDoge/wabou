//! Retained GPUI projection of completed Solid protocol frames.

// This is the internal frame-to-GPUI projection seam shared by the shell and
// runtime crates. It is not a stable application-facing API.
#![allow(missing_docs)]

use crate::gpui::Refineable as _;
use crate::{
    DirtyKind, NodeKey, ProjectedNodeKind, ProjectionError, ProjectionTree, StyleDiagnostic,
    StyleProjection, TextSelectionPolicy,
};
use wabou_style::{IrColor, IrLength, IrValue};

use wabou_protocol::{
    AtomPool, Frame, GRAPHIC_SOURCE_RESOURCE_RASTER, GRAPHIC_SOURCE_SVG, Op, ShadowValue,
    StyleValue, TEXT_BEHAVIOR_SINGLE_LINE,
};

#[derive(Clone, Debug, PartialEq)]
pub struct GpuiTextControl {
    pub key: NodeKey,
    pub kind: GpuiTextControlKind,
    pub language: Option<String>,
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub readonly: bool,
    pub submit_on_enter: bool,
    /// Revision of the last value explicitly authored by JavaScript.
    pub authored_value_revision: u64,
    /// Revision of the latest edit accepted by the native text control.
    pub native_edit_revision: u32,
    /// Latest native edit revision observed by JavaScript in a host frame.
    pub native_ack_revision: u32,
    pub style: GpuiTextControlStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuiTextControlKind {
    Input,
    Textarea,
    Editor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GpuiTextControlStyle {
    pub foreground: crate::gpui::Hsla,
    pub muted_foreground: crate::gpui::Hsla,
    pub background: crate::gpui::Hsla,
    pub border: crate::gpui::Hsla,
    pub selection: crate::gpui::Hsla,
    pub caret: crate::gpui::Hsla,
}

impl Default for GpuiTextControlStyle {
    fn default() -> Self {
        let foreground = crate::gpui::black();
        let mut selection = foreground;
        selection.alpha = 0.4;
        Self {
            foreground,
            muted_foreground: foreground,
            background: crate::gpui::transparent_black(),
            border: crate::gpui::transparent_black(),
            selection,
            caret: foreground,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GpuiNativeWidget {
    pub key: NodeKey,
    pub tag: crate::gpui::SharedString,
    pub attributes:
        std::collections::BTreeMap<crate::gpui::SharedString, crate::gpui::SharedString>,
    pub config: Option<crate::gpui::SharedString>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GpuiSelectableText {
    pub key: NodeKey,
    pub text: crate::gpui::SharedString,
    pub document_order: u64,
    pub select_all: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GpuiLayoutNode {
    pub key: NodeKey,
    pub kind: ProjectedNodeKind,
    pub parent: Option<NodeKey>,
    /// Whether this retained node participated in the latest completed native
    /// layout. Detached Solid nodes and descendants absorbed into a native
    /// text/widget element remain observable, but must not participate in
    /// locators, geometry diagnostics, or native semantics.
    pub attached: bool,
    pub attributes:
        std::collections::BTreeMap<crate::gpui::SharedString, crate::gpui::SharedString>,
    pub text: Option<crate::gpui::SharedString>,
    pub bounds: crate::gpui::Bounds<crate::gpui::Pixels>,
    pub content_bounds: crate::gpui::Bounds<crate::gpui::Pixels>,
    /// Completed single-line text geometry in logical window coordinates.
    pub text_metrics: Option<GpuiTextMetrics>,
    pub classes: Vec<String>,
    pub style_diagnostics: Vec<String>,
    pub listeners: Vec<u8>,
    pub focus_order: Option<i32>,
    pub pointer_events: bool,
    pub z_index: usize,
    pub overlay_plane: u8,
    pub widget: Option<String>,
    pub computed: GpuiComputedStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GpuiTextMetrics {
    /// `node` for ordinary projected text or `widget` for a native editor.
    pub source: &'static str,
    pub line_box: crate::gpui::Bounds<crate::gpui::Pixels>,
    pub baseline: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GpuiComputedStyle {
    pub position: String,
    pub overflow_x: String,
    pub overflow_y: String,
    pub font_size: Option<f32>,
    pub font_weight: Option<f32>,
    pub text_color: Option<crate::gpui::Hsla>,
    pub opacity: f32,
}

/// Active semantic color theme projected by the JavaScript stylesheet.
///
/// Native GPUI widgets consume the same palette as projected JSX instead of
/// falling back to a second, unrelated set of application defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuiThemeSnapshot {
    pub dark: bool,
    pub colors: std::collections::HashMap<String, u32>,
}

/// Imperative work whose semantics belong to the GPUI window rather than the
/// retained element tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GpuiCommand {
    Focus {
        id: NodeKey,
    },
    Blur {
        id: NodeKey,
    },
    ScrollTo {
        id: NodeKey,
        x: f32,
        y: f32,
    },
    ScrollBy {
        id: NodeKey,
        x: f32,
        y: f32,
    },
    SetTextSelection {
        id: NodeKey,
        anchor: u32,
        head: u32,
    },
    Text {
        id: NodeKey,
        command: GpuiTextCommand,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuiTextCommand {
    SelectAll,
    Undo,
    Redo,
}

#[derive(Debug)]
pub struct GpuiProjection {
    tree: ProjectionTree,
    stylesheet: Option<wabou_style::stylesheet::StyleSheet>,
    active_color_theme: Option<String>,
    active_theme_colors: Option<std::collections::HashMap<String, u32>>,
    classes: std::collections::HashMap<NodeKey, Vec<String>>,
    resolved_class_styles:
        std::collections::HashMap<Vec<String>, wabou_style::stylesheet::CascadeResult>,
    style_diagnostics: std::collections::HashMap<NodeKey, Vec<String>>,
    inline_styles: std::collections::HashMap<NodeKey, std::collections::BTreeMap<String, IrValue>>,
    layout_bounds: crate::element::ProjectedLayoutBounds,
    graphic_paint_states: crate::element::ProjectedGraphicPaintStates,
    pending_commands: Vec<GpuiCommand>,
    scroll_handles: std::collections::BTreeMap<NodeKey, crate::ProjectedScrollHandle>,
    uniform_list_handles: std::collections::BTreeMap<NodeKey, crate::gpui::UniformListScrollHandle>,
    protocol_gaps: std::collections::HashMap<NodeKey, std::collections::BTreeSet<&'static str>>,
    boundary_keys: std::rc::Rc<std::collections::BTreeSet<NodeKey>>,
    boundary_revisions: std::collections::BTreeMap<NodeKey, crate::ProjectionBoundaryRevision>,
    pending_boundary_changes: std::collections::BTreeMap<NodeKey, DirtyKind>,
    text_value_revisions: std::collections::BTreeMap<NodeKey, u64>,
    native_text_revisions: std::collections::BTreeMap<NodeKey, u32>,
    native_text_acks: std::collections::BTreeMap<NodeKey, u32>,
    next_text_value_revision: u64,
    #[cfg(test)]
    style_recomputation_count: usize,
}

/// Immutable, cheap-to-clone render view of one committed projection.
///
/// A GPUI `View` can retain this snapshot across root renders and only replace
/// it when the corresponding Wabou projection boundary is invalidated. The
/// node table and paint/layout handles remain shared rather than cloning the
/// projected UI tree.
#[derive(Clone)]
pub struct GpuiProjectionRenderSnapshot {
    tree: crate::ProjectionSnapshot,
    boundary_keys: std::rc::Rc<std::collections::BTreeSet<NodeKey>>,
    layout_bounds: crate::element::ProjectedLayoutBounds,
    graphic_paint_states: crate::element::ProjectedGraphicPaintStates,
    scroll_handles: std::rc::Rc<std::collections::BTreeMap<NodeKey, crate::ProjectedScrollHandle>>,
    uniform_list_handles:
        std::rc::Rc<std::collections::BTreeMap<NodeKey, crate::gpui::UniformListScrollHandle>>,
}

/// Runtime services used while materializing one retained projection boundary.
///
/// Keeping these handles together makes the JS-to-GPUI boundary explicit and
/// prevents the materialization API from growing one positional argument for
/// every native service it learns about.
pub struct GpuiProjectionElementContext {
    pub input: crate::ProjectedInputSink,
    pub focus: crate::gpui::FocusHandle,
    pub text_input: crate::ProjectedTextInputState,
    pub native: Option<crate::ProjectedNativeElementFactory>,
    pub subtree: Option<crate::ProjectedSubtreeElementFactory>,
    pub text_selections:
        std::rc::Rc<std::collections::BTreeMap<NodeKey, crate::ProjectedTextSelection>>,
}

impl GpuiProjectionRenderSnapshot {
    /// Layout contract used by a GPUI cached View which replaces this node in
    /// its parent. The cached View still renders the projected node itself;
    /// this refinement only lets the parent size and position the retained
    /// entity without materializing its contents first.
    pub fn projection_boundary_layout(
        &self,
        root: NodeKey,
    ) -> Option<crate::gpui::StyleRefinement> {
        let style = &self.tree.node(root)?.style;
        let mut layout = crate::gpui::StyleRefinement {
            display: Some(style.display),
            position: Some(style.position),
            aspect_ratio: style.aspect_ratio,
            align_self: style.align_self,
            flex_basis: Some(style.flex_basis),
            flex_grow: Some(style.flex_grow),
            flex_shrink: Some(style.flex_shrink),
            grid_location: style.grid_location.clone(),
            ..Default::default()
        };
        layout.inset.top = Some(style.inset.top);
        layout.inset.right = Some(style.inset.right);
        layout.inset.bottom = Some(style.inset.bottom);
        layout.inset.left = Some(style.inset.left);
        layout.size.width = Some(style.size.width);
        layout.size.height = Some(style.size.height);
        layout.min_size.width = Some(style.min_size.width);
        layout.min_size.height = Some(style.min_size.height);
        layout.max_size.width = Some(style.max_size.width);
        layout.max_size.height = Some(style.max_size.height);
        layout.margin.top = Some(style.margin.top);
        layout.margin.right = Some(style.margin.right);
        layout.margin.bottom = Some(style.margin.bottom);
        layout.margin.left = Some(style.margin.left);
        Some(layout)
    }

    pub fn projection_boundaries(&self) -> Vec<NodeKey> {
        self.boundary_keys.iter().copied().collect()
    }

    pub fn nearest_projection_boundary(&self, key: NodeKey) -> Option<NodeKey> {
        self.tree.nearest_projection_boundary(key)
    }

    /// Number of retained protocol nodes materialized by this boundary.
    /// Nested boundary roots are represented by one cached Entity in the
    /// parent and are therefore excluded from the parent's recursive work.
    pub fn projection_boundary_node_count(&self, root: NodeKey) -> usize {
        self.tree
            .keys()
            .filter(|key| self.tree.nearest_projection_boundary(*key) == Some(root))
            .count()
    }

    pub fn direct_projection_boundaries(&self, root: NodeKey) -> Vec<NodeKey> {
        self.boundary_keys
            .iter()
            .copied()
            .filter(|candidate| *candidate != root)
            .filter(|candidate| {
                let mut parent = self.tree.node(*candidate).and_then(|node| node.parent);
                while let Some(key) = parent {
                    if key == root {
                        return true;
                    }
                    let Some(node) = self.tree.node(key) else {
                        return false;
                    };
                    if self.boundary_keys.contains(&key) {
                        return false;
                    }
                    parent = node.parent;
                }
                false
            })
            .collect()
    }

    pub fn interactive_element(
        &self,
        root: NodeKey,
        context: GpuiProjectionElementContext,
    ) -> Result<crate::ProjectedElement, ProjectionError> {
        let GpuiProjectionElementContext {
            input,
            focus,
            text_input,
            native,
            subtree,
            text_selections,
        } = context;
        crate::ProjectedElement::from_tree(
            self.tree.clone(),
            root,
            crate::element::ProjectedElementContext {
                input: Some(input),
                root_focus: Some(focus),
                text_input: Some(text_input),
                native,
                subtree,
                layout_bounds: Some(self.layout_bounds.clone()),
                graphic_paint_states: Some(self.graphic_paint_states.clone()),
                scroll_handles: Some(self.scroll_handles.clone()),
                uniform_list_handles: Some(self.uniform_list_handles.clone()),
                text_selections: Some(text_selections),
                text_selection_policy: None,
            },
            false,
        )
    }
}

impl Default for GpuiProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuiProjection {
    pub fn new() -> Self {
        let mut tree = ProjectionTree::default();
        tree.insert(
            NodeKey::ROOT,
            None,
            0,
            gpui_root_style(),
            None,
            ProjectedNodeKind::Root,
        )
        .expect("the canonical projection root is unique");
        let _ = tree.commit();
        let mut scroll_handles = std::collections::BTreeMap::new();
        scroll_handles.insert(NodeKey::ROOT, crate::ProjectedScrollHandle::default());
        Self {
            tree,
            stylesheet: None,
            active_color_theme: None,
            active_theme_colors: None,
            classes: std::collections::HashMap::new(),
            resolved_class_styles: std::collections::HashMap::new(),
            style_diagnostics: std::collections::HashMap::new(),
            inline_styles: std::collections::HashMap::new(),
            layout_bounds: Default::default(),
            graphic_paint_states: Default::default(),
            pending_commands: Vec::new(),
            scroll_handles,
            uniform_list_handles: Default::default(),
            protocol_gaps: std::collections::HashMap::new(),
            boundary_keys: std::rc::Rc::new(std::collections::BTreeSet::from([NodeKey::ROOT])),
            boundary_revisions: std::collections::BTreeMap::from([(
                NodeKey::ROOT,
                crate::ProjectionBoundaryRevision::default(),
            )]),
            pending_boundary_changes: std::collections::BTreeMap::new(),
            text_value_revisions: std::collections::BTreeMap::new(),
            native_text_revisions: std::collections::BTreeMap::new(),
            native_text_acks: std::collections::BTreeMap::new(),
            next_text_value_revision: 0,
            #[cfg(test)]
            style_recomputation_count: 0,
        }
    }

    /// Apply the structural part of one Solid flush without publishing it.
    ///
    /// Structural mutations and authored styles are accumulated before
    /// `finish_frame` publishes the retained snapshot. GPUI therefore never
    /// observes a newly attached node with the previous/default style.
    pub fn apply_ops(
        &mut self,
        frame: &Frame<'_>,
        atoms: &AtomPool,
        mut resolve_raster: impl FnMut(&str) -> Option<std::sync::Arc<crate::gpui::Image>>,
    ) -> Result<(), ProjectionError> {
        // A Solid commit is already atomic. Resolve the final cascade once per
        // node instead of rebuilding the complete GPUI style after every
        // class, color, size, shadow, and inline-style record in the batch.
        let mut dirty_styles = std::collections::BTreeSet::new();
        for op in &frame.ops {
            match op {
                Op::CreateElement { id, tag } => {
                    let tag = atoms.resolve(*tag).unwrap_or("unknown");
                    self.tree.insert_detached(
                        *id,
                        gpui_style(),
                        None,
                        ProjectedNodeKind::Element(tag.into()),
                    )?;
                    self.scroll_handles.entry(*id).or_default();
                    if tag == "virtual-list" {
                        self.uniform_list_handles.entry(*id).or_default();
                    }
                }
                Op::CreateText { id, text } => {
                    self.tree.insert_detached(
                        *id,
                        gpui_style(),
                        Some((*text).into()),
                        ProjectedNodeKind::Text,
                    )?;
                    self.scroll_handles.entry(*id).or_default();
                }
                Op::AppendChild { parent, child } => {
                    let index = self
                        .tree
                        .node(*parent)
                        .ok_or(ProjectionError::MissingParent(*parent))?
                        .children
                        .len();
                    self.tree.attach_child(*child, *parent, index)?;
                }
                Op::InsertBefore {
                    parent,
                    child,
                    ref_id,
                } => {
                    let index = self
                        .tree
                        .node(*parent)
                        .ok_or(ProjectionError::MissingParent(*parent))?
                        .children
                        .iter()
                        .position(|candidate| candidate == ref_id)
                        .ok_or(ProjectionError::MissingNode(*ref_id))?;
                    self.tree.attach_child(*child, *parent, index)?;
                }
                Op::RemoveChild { child, .. } => self.tree.detach(*child)?,
                Op::SetText { id, text } => {
                    self.tree.update_text(*id, Some((*text).into()))?;
                }
                Op::SetWidgetConfig { id, json } => {
                    self.tree.update_widget_config(*id, Some((*json).into()))?;
                }
                Op::RemoveWidgetConfig { id } => {
                    self.tree.update_widget_config(*id, None)?;
                }
                Op::SetTextBehavior { id, flags } => {
                    self.tree.update_text_behavior(*id, *flags)?;
                }
                Op::SetTextMaxLines { id, max_lines } => {
                    self.tree.update_text_max_lines(*id, *max_lines)?;
                }
                Op::SetAttribute { id, name, value } => {
                    if let Some(name) = atoms.resolve(*name) {
                        if name == "value" {
                            self.tree
                                .update_attribute(*id, name.into(), (*value).into())?;
                            self.record_authored_text_value(*id);
                            continue;
                        }
                        self.tree
                            .update_attribute(*id, name.into(), (*value).into())?;
                    }
                }
                Op::RemoveAttribute { id, name } => {
                    if let Some(name) = atoms.resolve(*name) {
                        if name == "value" {
                            self.tree.remove_attribute(*id, name)?;
                            self.record_authored_text_value(*id);
                            continue;
                        }
                        self.tree.remove_attribute(*id, name)?;
                    }
                }
                Op::SetStyleValue { id, prop, value } => {
                    if let Some(property) = atoms.resolve(*prop) {
                        self.inline_styles
                            .entry(*id)
                            .or_default()
                            .insert(property.to_owned(), protocol_style_value(*value));
                        dirty_styles.insert(*id);
                    }
                }
                Op::SetStyle { id, prop, value } => {
                    if let Some(property) = atoms.resolve(*prop) {
                        self.inline_styles
                            .entry(*id)
                            .or_default()
                            .insert(property.to_owned(), wabou_style::parse_inline_value(value));
                        dirty_styles.insert(*id);
                    }
                }
                Op::SetShadows { id, shadows } => {
                    self.inline_styles.entry(*id).or_default().insert(
                        "box-shadow".to_owned(),
                        IrValue::List {
                            values: shadows.iter().copied().map(protocol_shadow_value).collect(),
                        },
                    );
                    dirty_styles.insert(*id);
                }
                Op::RemoveStyle { id, prop } => {
                    if let Some(property) = atoms.resolve(*prop) {
                        let remove_entry = if let Some(styles) = self.inline_styles.get_mut(id) {
                            styles.remove(property);
                            styles.is_empty()
                        } else {
                            false
                        };
                        if remove_entry {
                            self.inline_styles.remove(id);
                        }
                        dirty_styles.insert(*id);
                    }
                }
                Op::SetClassName { id, classes } => {
                    self.classes.insert(
                        *id,
                        classes
                            .iter()
                            .filter_map(|class| atoms.resolve(*class).map(str::to_owned))
                            .collect(),
                    );
                    dirty_styles.insert(*id);
                }
                Op::AddEventListener { id, event_type } => {
                    self.tree.add_event_listener(*id, *event_type)?;
                }
                Op::RemoveEventListener { id, event_type } => {
                    self.tree.remove_event_listener(*id, *event_type)?;
                }
                Op::SetInteractionPolicy {
                    id,
                    flags,
                    focus_order,
                } => {
                    self.tree.update_interaction_policy(
                        *id,
                        (flags & wabou_protocol::INTERACTION_POLICY_FOCUSABLE != 0)
                            .then_some(*focus_order),
                        flags & wabou_protocol::INTERACTION_POLICY_BLOCK_SUBTREE != 0,
                        flags & wabou_protocol::INTERACTION_POLICY_CONTAIN_FOCUS != 0,
                    )?;
                }
                Op::DropNode { id } => {
                    // The JS owner releases one generational handle per node. A
                    // parent release recursively retires the retained host subtree,
                    // so later child release records in the same sweep are expected.
                    // Keep all other mutations strict; only resource release is
                    // deliberately idempotent.
                    if self.tree.node(*id).is_none() {
                        continue;
                    }
                    let removed = self.tree.remove(*id)?;
                    for key in removed {
                        if key != NodeKey::ROOT {
                            std::rc::Rc::make_mut(&mut self.boundary_keys).remove(&key);
                        }
                        dirty_styles.remove(&key);
                        self.layout_bounds.borrow_mut().remove(&key);
                        self.graphic_paint_states.borrow_mut().remove(&key);
                        self.inline_styles.remove(&key);
                        self.classes.remove(&key);
                        self.style_diagnostics.remove(&key);
                        self.protocol_gaps.remove(&key);
                        self.scroll_handles.remove(&key);
                        self.uniform_list_handles.remove(&key);
                        self.text_value_revisions.remove(&key);
                        self.native_text_revisions.remove(&key);
                        self.native_text_acks.remove(&key);
                    }
                }
                Op::AcknowledgeTextValue { id, revision } => {
                    self.tree
                        .node(*id)
                        .ok_or(ProjectionError::MissingNode(*id))?;
                    self.native_text_acks.insert(*id, *revision);
                }
                Op::SetGraphicSource { id, kind, source } => match *kind {
                    GRAPHIC_SOURCE_SVG => {
                        self.graphic_paint_states.borrow_mut().remove(id);
                        self.tree.update_svg_source(
                            *id,
                            Some(std::sync::Arc::from(source.as_bytes())),
                        )?;
                    }
                    GRAPHIC_SOURCE_RESOURCE_RASTER => {
                        self.tree.update_image(*id, resolve_raster(source))?;
                    }
                    _ => self.record_protocol_gap(*id, "unsupported graphic source kind"),
                },
                Op::ClearGraphicSource { id, kind } if *kind == GRAPHIC_SOURCE_SVG => {
                    self.graphic_paint_states.borrow_mut().remove(id);
                    self.tree.update_svg_source(*id, None)?
                }
                Op::ClearGraphicSource { id, kind } if *kind == GRAPHIC_SOURCE_RESOURCE_RASTER => {
                    self.tree.update_image(*id, None)?
                }
                Op::ClearGraphicSource { id, .. } => {
                    self.record_protocol_gap(*id, "unsupported graphic source kind");
                }
                Op::SetGraphicData { id, kind, data }
                    if *kind == wabou_protocol::GRAPHIC_DATA_VECTOR_PATH =>
                {
                    let path = crate::vector_path::ProjectedVectorPath::decode(data)
                        .map_err(|_| ProjectionError::InvalidGraphicData(*id))?;
                    self.tree
                        .update_vector_path(*id, Some(std::sync::Arc::new(path)))?;
                }
                Op::SetGraphicData { id, .. } => {
                    self.record_protocol_gap(*id, "unsupported graphic data kind");
                }
                Op::ClearGraphicData { id, kind }
                    if *kind == wabou_protocol::GRAPHIC_DATA_VECTOR_PATH =>
                {
                    self.tree.update_vector_path(*id, None)?;
                }
                Op::ClearGraphicData { id, .. } => {
                    self.record_protocol_gap(*id, "unsupported graphic data kind");
                }
                Op::SetTransform2D { id, matrix } => {
                    self.tree.update_transform(*id, *matrix)?;
                }
                Op::SetOverlayPlane { id, plane } => {
                    self.tree.update_overlay_plane(*id, *plane)?;
                }
                Op::SetProjectionBoundary { id, enabled } => {
                    self.tree.update_projection_boundary(*id, *enabled)?;
                    if *id != NodeKey::ROOT {
                        if *enabled {
                            std::rc::Rc::make_mut(&mut self.boundary_keys).insert(*id);
                        } else {
                            std::rc::Rc::make_mut(&mut self.boundary_keys).remove(id);
                        }
                    }
                }
                Op::SetScrollbarStyle {
                    id,
                    visibility,
                    hide_delay,
                    fade_duration,
                    thickness,
                    margin,
                    min_thumb_length,
                    radius,
                    colors,
                } => self.tree.update_scrollbar_style(
                    *id,
                    crate::tree::ProjectedScrollbarStyle {
                        visibility: *visibility,
                        hide_delay: *hide_delay,
                        fade_duration: *fade_duration,
                        thickness: *thickness,
                        margin: *margin,
                        min_thumb_length: *min_thumb_length,
                        radius: *radius,
                        colors: *colors,
                    },
                )?,
                Op::FocusNode { id } => self.pending_commands.push(GpuiCommand::Focus { id: *id }),
                Op::BlurNode { id } => self.pending_commands.push(GpuiCommand::Blur { id: *id }),
                Op::ScrollTo { id, x, y } => self.pending_commands.push(GpuiCommand::ScrollTo {
                    id: *id,
                    x: *x,
                    y: *y,
                }),
                Op::ScrollBy { id, x, y } => self.pending_commands.push(GpuiCommand::ScrollBy {
                    id: *id,
                    x: *x,
                    y: *y,
                }),
                Op::SetTextSelection { id, anchor, head } => {
                    self.pending_commands.push(GpuiCommand::SetTextSelection {
                        id: *id,
                        anchor: *anchor,
                        head: *head,
                    });
                }
                Op::TextCommand { id, command } => {
                    let command = match command {
                        1 => GpuiTextCommand::SelectAll,
                        2 => GpuiTextCommand::Undo,
                        3 => GpuiTextCommand::Redo,
                        _ => unreachable!("protocol validates text commands"),
                    };
                    self.pending_commands
                        .push(GpuiCommand::Text { id: *id, command });
                }
            }
        }
        for key in dirty_styles {
            self.recompute_style(key)?;
        }
        Ok(())
    }

    fn record_protocol_gap(&mut self, id: NodeKey, operation: &'static str) {
        self.protocol_gaps.entry(id).or_default().insert(operation);
    }

    /// Drain imperative commands after the completed protocol frame has been
    /// projected. The GPUI view executes them against live focus/scroll state.
    pub fn take_commands(&mut self) -> Vec<GpuiCommand> {
        std::mem::take(&mut self.pending_commands)
    }

    pub fn apply_scroll_command(
        &self,
        command: GpuiCommand,
    ) -> Option<crate::ProjectedScrollEvent> {
        let (id, changed) = match command {
            GpuiCommand::ScrollTo { id, x, y } => {
                (id, self.scroll_handles.get(&id)?.scroll_to(x, y))
            }
            GpuiCommand::ScrollBy { id, x, y } => {
                (id, self.scroll_handles.get(&id)?.scroll_by(x, y))
            }
            GpuiCommand::Focus { .. }
            | GpuiCommand::Blur { .. }
            | GpuiCommand::SetTextSelection { .. }
            | GpuiCommand::Text { .. } => return None,
        };
        changed.then(|| {
            let position = self.scroll_handles[&id].position();
            crate::ProjectedScrollEvent {
                target: id,
                x: position.x.into(),
                y: position.y.into(),
            }
        })
    }

    /// Apply a synthetic wheel delta using the same nearest-scrollable-
    /// ancestor policy as the interactive GPUI element tree.
    ///
    /// Native GPUI wheel delivery updates the retained scroll handle inside
    /// `ProjectedElement`. Headless behavior tests bypass that element event
    /// callback, so they use this seam instead of pretending that a guest
    /// `wheel` listener is responsible for native scrolling.
    pub fn apply_wheel_delta(
        &self,
        target: NodeKey,
        delta_x: f32,
        delta_y: f32,
    ) -> Option<crate::ProjectedScrollEvent> {
        let mut current = Some(target);
        while let Some(key) = current {
            let node = self.tree.node(key)?;
            let scroll_x = node.style.overflow.x == crate::gpui::Overflow::Scroll;
            let scroll_y = node.style.overflow.y == crate::gpui::Overflow::Scroll;
            if (scroll_x || scroll_y)
                && self.scroll_handles.get(&key).is_some_and(|handle| {
                    handle.scroll_by(
                        if scroll_x { delta_x } else { 0.0 },
                        if scroll_y { delta_y } else { 0.0 },
                    )
                })
            {
                let position = self.scroll_handles[&key].position();
                return Some(crate::ProjectedScrollEvent {
                    target: key,
                    x: position.x.into(),
                    y: position.y.into(),
                });
            }
            current = node.parent;
        }
        None
    }

    /// Return explicit formal-runtime gaps. This exists to make unsupported
    /// protocol semantics observable in tests and DevTools rather than being
    /// silently accepted by a backend wildcard.
    pub fn protocol_gaps(&self) -> Vec<(NodeKey, &'static str)> {
        let mut gaps = self
            .protocol_gaps
            .iter()
            .flat_map(|(key, operations)| operations.iter().map(|operation| (*key, *operation)))
            .collect::<Vec<_>>();
        gaps.sort_unstable_by_key(|(key, operation)| (*key, *operation));
        gaps
    }

    /// Publish structure, text, and resolved-style changes as one GPUI update.
    #[must_use]
    pub fn finish_frame_profiled(&mut self) -> crate::ProjectionInvalidationStats {
        self.layout_bounds
            .borrow_mut()
            .retain(|key, _| self.tree.node(*key).is_some());
        let pending = self.tree.commit();
        let snapshot = self.tree.snapshot();
        self.boundary_revisions
            .retain(|key, _| self.boundary_keys.contains(key));
        for key in self.boundary_keys.iter().copied() {
            self.boundary_revisions.entry(key).or_default();
        }
        let mut frame_boundary_changes = std::collections::BTreeMap::<NodeKey, DirtyKind>::new();
        for pending_node in &pending {
            let Some(boundary) = snapshot.nearest_projection_boundary(pending_node.key) else {
                continue;
            };
            frame_boundary_changes
                .entry(boundary)
                .and_modify(|dirty| *dirty |= pending_node.dirty)
                .or_insert(pending_node.dirty);

            // A boundary root is also a layout child of its owning boundary.
            // Its contents are materialized by its own Entity, but the parent
            // captures the child's `StyleRefinement` when it installs the
            // cached Entity. Propagate layout-affecting root changes one level
            // so that placement is refreshed without invalidating siblings or
            // walking the boundary's descendants in the parent.
            if boundary == pending_node.key
                && pending_node
                    .dirty
                    .intersects(DirtyKind::STRUCTURE | DirtyKind::LAYOUT | DirtyKind::TEXT)
                && let Some(parent) = snapshot.node(boundary).and_then(|node| node.parent)
                && let Some(owner) = snapshot.nearest_projection_boundary(parent)
            {
                frame_boundary_changes
                    .entry(owner)
                    .and_modify(|dirty| *dirty |= DirtyKind::LAYOUT)
                    .or_insert(DirtyKind::LAYOUT);
            }
        }
        for (boundary, dirty) in frame_boundary_changes {
            self.boundary_revisions
                .entry(boundary)
                .or_default()
                .invalidate(dirty);
            self.pending_boundary_changes
                .entry(boundary)
                .and_modify(|pending| *pending |= dirty)
                .or_insert(dirty);
        }
        crate::ProjectionInvalidationStats::from_pending(self.tree.revision(), &pending)
    }

    /// Drain the exact explicit boundary owners invalidated since the previous
    /// native synchronization pass.
    pub fn take_boundary_changes(&mut self) -> std::collections::BTreeMap<NodeKey, DirtyKind> {
        std::mem::take(&mut self.pending_boundary_changes)
    }

    /// Revision clocks for every explicit retained projection boundary.
    pub fn projection_boundary_revisions(
        &self,
    ) -> &std::collections::BTreeMap<NodeKey, crate::ProjectionBoundaryRevision> {
        &self.boundary_revisions
    }

    /// Current revision clocks for one retained boundary owner.
    pub fn projection_boundary_revision(
        &self,
        key: NodeKey,
    ) -> Option<crate::ProjectionBoundaryRevision> {
        self.boundary_revisions.get(&key).copied()
    }

    /// Publish structure, text, and resolved-style changes as one GPUI update.
    #[must_use]
    pub fn finish_frame(&mut self) -> bool {
        self.finish_frame_profiled().changed()
    }

    #[doc(hidden)]
    pub fn revision(&self) -> u64 {
        self.tree.revision()
    }

    pub fn contains(&self, key: NodeKey) -> bool {
        self.tree.node(key).is_some()
    }

    /// Whether an attached projected node participates in keyboard focus.
    #[must_use]
    pub fn is_focusable(&self, key: NodeKey) -> bool {
        self.tree
            .node(key)
            .is_some_and(|node| node.attached && node.focus_order.is_some())
    }

    /// Number of nodes retained by the canonical GPUI projection.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.tree.len()
    }

    /// Whether `target` or one of its retained ancestors listens for `event_type`.
    ///
    /// Event bubbling is a guest-runtime concern; the native shell only needs
    /// this query to avoid crossing the JS boundary when no handler can run.
    pub fn has_listener_in_chain(&self, mut target: NodeKey, event_type: u8) -> bool {
        loop {
            let Some(node) = self.tree.node(target) else {
                return false;
            };
            if node.listeners.contains(&event_type) {
                return true;
            }
            let Some(parent) = node.parent else {
                return false;
            };
            target = parent;
        }
    }

    pub fn text_controls(&self) -> Vec<GpuiTextControl> {
        self.tree
            .roots()
            .iter()
            .flat_map(|root| self.text_controls_below(*root))
            .collect()
    }

    pub fn update_authored_attribute(
        &mut self,
        key: NodeKey,
        name: &str,
        value: &str,
    ) -> Result<(), ProjectionError> {
        self.tree.update_attribute(key, name.into(), value.into())?;
        if name == "value" {
            self.record_authored_text_value(key);
        }
        Ok(())
    }

    /// Apply a native editor mutation without presenting it as a new
    /// JavaScript-authored controlled value.
    pub fn update_native_text_value(
        &mut self,
        key: NodeKey,
        value: &str,
        revision: u32,
    ) -> Result<(), ProjectionError> {
        self.tree
            .update_attribute(key, "value".into(), value.into())?;
        self.native_text_revisions.insert(key, revision);
        Ok(())
    }

    fn record_authored_text_value(&mut self, key: NodeKey) {
        self.next_text_value_revision = self.next_text_value_revision.wrapping_add(1).max(1);
        self.text_value_revisions
            .insert(key, self.next_text_value_revision);
    }

    pub fn native_widgets(&self, mut accepts: impl FnMut(&str) -> bool) -> Vec<GpuiNativeWidget> {
        let mut widgets = Vec::new();
        let mut pending = self.tree.roots().to_vec();
        while let Some(key) = pending.pop() {
            let Some(node) = self.tree.node(key) else {
                continue;
            };
            pending.extend(node.children.iter().rev().copied());
            let ProjectedNodeKind::Element(tag) = &node.kind else {
                continue;
            };
            if accepts(tag.as_ref()) {
                widgets.push(GpuiNativeWidget {
                    key,
                    tag: tag.clone(),
                    attributes: node.attributes.clone(),
                    config: node.widget_config.clone(),
                });
            }
        }
        widgets
    }

    fn text_controls_below(&self, root: NodeKey) -> Vec<GpuiTextControl> {
        let mut controls = Vec::new();
        let mut pending = vec![root];
        while let Some(key) = pending.pop() {
            let Some(node) = self.tree.node(key) else {
                continue;
            };
            pending.extend(node.children.iter().rev().copied());
            let ProjectedNodeKind::Element(tag) = &node.kind else {
                continue;
            };
            let kind = match tag.as_ref() {
                "input" => GpuiTextControlKind::Input,
                "textarea" => GpuiTextControlKind::Textarea,
                "editor" => GpuiTextControlKind::Editor,
                _ => continue,
            };
            let theme_color = |name: &str| {
                self.active_theme_colors
                    .as_ref()?
                    .get(name)
                    .copied()
                    .map(crate::gpui::rgba)
                    .map(crate::gpui::rgb_to_hsla)
            };
            let foreground = node
                .style
                .text
                .color
                .or_else(|| theme_color("primary"))
                .unwrap_or_else(crate::gpui::black);
            let mut selection = theme_color("accent").unwrap_or(foreground);
            selection.alpha = 0.4;
            controls.push(GpuiTextControl {
                key,
                kind,
                language: node.attributes.get("language").map(ToString::to_string),
                value: node
                    .attributes
                    .get("value")
                    .map_or_else(String::new, ToString::to_string),
                placeholder: node
                    .attributes
                    .get("placeholder")
                    .map_or_else(String::new, ToString::to_string),
                disabled: node.attributes.contains_key("disabled"),
                readonly: node.attributes.contains_key("readonly")
                    || node.attributes.contains_key("readOnly"),
                submit_on_enter: node.attributes.contains_key("submitOnEnter"),
                authored_value_revision: self
                    .text_value_revisions
                    .get(&key)
                    .copied()
                    .unwrap_or_default(),
                native_edit_revision: self
                    .native_text_revisions
                    .get(&key)
                    .copied()
                    .unwrap_or_default(),
                native_ack_revision: self.native_text_acks.get(&key).copied().unwrap_or_default(),
                style: GpuiTextControlStyle {
                    foreground,
                    muted_foreground: theme_color("muted").unwrap_or(foreground),
                    background: node
                        .style
                        .background
                        .as_ref()
                        .and_then(crate::gpui::Fill::color)
                        .and_then(|background| background.as_solid())
                        .or_else(|| theme_color("input"))
                        .unwrap_or_else(crate::gpui::transparent_black),
                    border: node
                        .style
                        .border_color
                        .or_else(|| theme_color("strong"))
                        .unwrap_or_else(crate::gpui::transparent_black),
                    selection,
                    caret: theme_color("focus").unwrap_or(foreground),
                },
            });
        }
        controls
    }

    #[doc(hidden)]
    pub fn tree_element(&self, root: NodeKey) -> Result<crate::ProjectedElement, ProjectionError> {
        self.tree.element(root)
    }

    pub fn interactive_tree_element(
        &self,
        root: NodeKey,
        input: crate::ProjectedInputSink,
        focus: crate::gpui::FocusHandle,
        text_input: crate::ProjectedTextInputState,
        native: Option<crate::ProjectedNativeElementFactory>,
        text_selections: std::rc::Rc<
            std::collections::BTreeMap<NodeKey, crate::ProjectedTextSelection>,
        >,
    ) -> Result<crate::ProjectedElement, ProjectionError> {
        self.render_snapshot().interactive_element(
            root,
            GpuiProjectionElementContext {
                input,
                focus,
                text_input,
                native,
                subtree: None,
                text_selections,
            },
        )
    }

    /// Freeze the current committed projection for a retained GPUI View.
    #[must_use]
    pub fn render_snapshot(&self) -> GpuiProjectionRenderSnapshot {
        GpuiProjectionRenderSnapshot {
            tree: self.tree.snapshot(),
            boundary_keys: self.boundary_keys.clone(),
            layout_bounds: self.layout_bounds.clone(),
            graphic_paint_states: self.graphic_paint_states.clone(),
            scroll_handles: std::rc::Rc::new(self.scroll_handles.clone()),
            uniform_list_handles: std::rc::Rc::new(self.uniform_list_handles.clone()),
        }
    }

    pub fn selectable_texts(&self) -> Vec<GpuiSelectableText> {
        fn visit(
            tree: &crate::ProjectionSnapshot,
            key: NodeKey,
            inherited: TextSelectionPolicy,
            order: &mut u64,
            output: &mut Vec<GpuiSelectableText>,
        ) {
            let Some(node) = tree.node(key) else {
                return;
            };
            let policy = node.text_selection.unwrap_or(inherited);
            let is_native_editor = matches!(
                &node.kind,
                ProjectedNodeKind::Element(tag)
                    if matches!(tag.as_ref(), "input" | "textarea" | "password-input" | "editor")
            );
            if policy != TextSelectionPolicy::None
                && !is_native_editor
                && let Some(text) = crate::element::projected_text(tree, node)
            {
                output.push(GpuiSelectableText {
                    key,
                    text,
                    document_order: *order,
                    select_all: policy == TextSelectionPolicy::All,
                });
                *order += 1;
            }
            for child in &node.children {
                if node.text_behavior & wabou_protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT != 0
                    && tree
                        .node(*child)
                        .is_some_and(|child| child.kind == ProjectedNodeKind::Text)
                {
                    continue;
                }
                visit(tree, *child, policy, order, output);
            }
        }

        let tree = self.tree.snapshot();
        let mut output = Vec::new();
        let mut order = 0;
        visit(
            &tree,
            NodeKey::ROOT,
            TextSelectionPolicy::None,
            &mut order,
            &mut output,
        );
        output
    }

    pub fn layout_snapshot(&self) -> Vec<GpuiLayoutNode> {
        let bounds = self.layout_bounds.borrow();
        let tree = self.tree.snapshot();

        fn resolved_text_style(
            tree: &crate::ProjectionSnapshot,
            key: NodeKey,
        ) -> crate::gpui::TextStyle {
            let mut path = Vec::new();
            let mut current = Some(key);
            while let Some(key) = current {
                let Some(node) = tree.node(key) else {
                    break;
                };
                path.push(key);
                current = node.parent;
            }
            let mut text = crate::gpui::TextStyle::default();
            for key in path.into_iter().rev() {
                if let Some(refinement) = tree.node(key).and_then(|node| node.style.text_style()) {
                    text.refine(refinement);
                }
            }
            text
        }

        self.tree
            .keys()
            .filter_map(|key| {
                let node = self.tree.node(key)?;
                let resolved_text = resolved_text_style(&tree, key);
                let node_bounds = bounds.get(&key).copied().unwrap_or_default();
                let padding = node
                    .style
                    .padding
                    .to_pixels(node_bounds.size.into(), crate::gpui::px(16.0));
                let content_bounds = crate::gpui::Bounds::new(
                    node_bounds.origin + crate::gpui::point(padding.left, padding.top),
                    crate::gpui::size(
                        (node_bounds.size.width - padding.left - padding.right)
                            .max(crate::gpui::Pixels::ZERO),
                        (node_bounds.size.height - padding.top - padding.bottom)
                            .max(crate::gpui::Pixels::ZERO),
                    ),
                );
                // `attached` in the public layout snapshot means the node
                // participated in the completed native layout, not merely
                // that it remains reachable in Solid's retained tree. GPUI
                // intentionally absorbs raw text and `text-span` descendants
                // into their owning `StyledText`; exposing those descendants
                // as visible 0x0 boxes creates false overflow and hit-test
                // diagnostics.
                let retained_attached = if node.attached
                    && node.style.display != crate::gpui::Display::None
                {
                    let mut parent = node.parent;
                    let mut reachable = true;
                    while let Some(parent_key) = parent {
                        let Some(ancestor) = tree.node(parent_key) else {
                            reachable = false;
                            break;
                        };
                        if !ancestor.attached
                            || ancestor.style.display == crate::gpui::Display::None
                        {
                            reachable = false;
                            break;
                        }
                        parent = ancestor.parent;
                    }
                    reachable
                } else {
                    false
                };
                let attached = retained_attached && bounds.contains_key(&key);
                let mut style_diagnostics = self
                    .style_diagnostics
                    .get(&key)
                    .cloned()
                    .unwrap_or_default();
                if node.svg_source.is_none() && !is_translation_matrix(node.transform) {
                    style_diagnostics.push(format!(
                        "GPUI retained unsupported affine transform {:?}; only translation is painted",
                        node.transform
                    ));
                }
                if retained_attached
                    && matches!(
                    &node.kind,
                    ProjectedNodeKind::Element(tag) if tag.as_ref() == "svg"
                )
                {
                    match &node.svg_source {
                        None => style_diagnostics
                            .push("GPUI SVG node has no projected source".to_owned()),
                        Some(source) if source.bytes.is_empty() => style_diagnostics
                            .push("GPUI SVG source is empty".to_owned()),
                        Some(_)
                            if f32::from(node_bounds.size.width) <= 0.0
                                || f32::from(node_bounds.size.height) <= 0.0 =>
                        {
                            style_diagnostics.push(format!(
                                "GPUI SVG source cannot paint into {:.1}x{:.1} bounds",
                                f32::from(node_bounds.size.width),
                                f32::from(node_bounds.size.height)
                            ));
                        }
                        Some(_) => {}
                    }
                }
                if let Some(gaps) = self.protocol_gaps.get(&key) {
                    style_diagnostics.extend(
                        gaps.iter()
                            .map(|gap| format!("unsupported by formal GPUI runtime: {gap}")),
                    );
                }
                match self.graphic_paint_states.borrow().get(&key) {
                    Some(crate::element::ProjectedGraphicPaintState::SvgFailed { message }) => {
                        style_diagnostics.push(format!("GPUI failed to paint SVG: {message}"));
                    }
                    Some(crate::element::ProjectedGraphicPaintState::SvgPainted {
                        color,
                        ..
                    }) if color.alpha <= 0.0 => {
                        style_diagnostics.push(
                            "GPUI painted SVG with a fully transparent inherited color".to_owned(),
                        );
                    }
                    _ => {}
                }
                let text = crate::element::projected_text(&tree, node);
                let native_single_line_editor = matches!(
                    &node.kind,
                    ProjectedNodeKind::Element(tag)
                        if matches!(tag.as_ref(), "input" | "password-input")
                );
                let line_box = if native_single_line_editor {
                    Some(content_bounds)
                } else if text.as_ref().is_some_and(|text| !text.is_empty()) {
                    Some(node_bounds)
                } else {
                    None
                };
                let text_metrics = line_box
                    .filter(|line_box| {
                        line_box.size.width > crate::gpui::Pixels::ZERO
                            && line_box.size.height > crate::gpui::Pixels::ZERO
                    })
                    .map(|line_box| {
                        let font_size = f32::from(
                            resolved_text.font_size.to_pixels(crate::gpui::px(16.0)),
                        );
                        let line_top = f32::from(line_box.origin.y);
                        let line_height = f32::from(line_box.size.height);
                        GpuiTextMetrics {
                            source: if native_single_line_editor {
                                "widget"
                            } else {
                                "node"
                            },
                            line_box,
                            // GPUI centers the em box inside the resolved line box.
                            // A baseline at 0.8em matches GPUI's ordinary Latin
                            // ascent and, crucially, uses the native editor's
                            // content box rather than its padded control surface.
                            baseline: line_top
                                + (line_height - font_size) / 2.0
                                + font_size * 0.8,
                        }
                    });
                Some(GpuiLayoutNode {
                    key,
                    kind: node.kind.clone(),
                    parent: node.parent,
                    attached,
                    attributes: node.attributes.clone(),
                    // Publish the same effective string GPUI shapes. Direct
                    // protocol text leaves remain in the snapshot for exact
                    // identity, while their aggregate owner is now queryable
                    // by the visible text users actually see.
                    text,
                    // Retained but non-materialized nodes remain observable
                    // with a zero rectangle and `attached: false`.
                    bounds: node_bounds,
                    content_bounds,
                    text_metrics,
                    classes: self.classes.get(&key).cloned().unwrap_or_default(),
                    style_diagnostics,
                    listeners: node.listeners.iter().copied().collect(),
                    focus_order: node.focus_order,
                    pointer_events: node.pointer_events,
                    z_index: node.z_index,
                    overlay_plane: node.overlay_plane,
                    widget: node
                        .widget_config
                        .as_ref()
                        .and_then(|_| match &node.kind {
                            ProjectedNodeKind::Element(tag) => Some(tag.to_string()),
                            ProjectedNodeKind::Root | ProjectedNodeKind::Text => None,
                        }),
                    computed: GpuiComputedStyle {
                        position: format!("{:?}", node.style.position),
                        overflow_x: format!("{:?}", node.style.overflow.x),
                        overflow_y: format!("{:?}", node.style.overflow.y),
                        font_size: Some(f32::from(
                            resolved_text.font_size.to_pixels(crate::gpui::px(16.0)),
                        )),
                        font_weight: Some(resolved_text.font_weight.0),
                        text_color: Some(resolved_text.color),
                        opacity: node.style.opacity.unwrap_or(1.0),
                    },
                })
            })
            .collect()
    }

    pub fn update_style(
        &mut self,
        key: NodeKey,
        style: crate::gpui::Style,
    ) -> Result<(), ProjectionError> {
        let previous = &self
            .tree
            .node(key)
            .ok_or(ProjectionError::MissingNode(key))?
            .style;
        let dirty = projected_style_dirty_kind(previous, &style);
        if dirty.is_empty() {
            return Ok(());
        }
        self.tree.update_style(key, style, dirty)
    }

    pub fn apply_style_declaration(
        &mut self,
        key: NodeKey,
        property: &str,
        value: &IrValue,
    ) -> Result<Option<StyleDiagnostic>, ProjectionError> {
        let current = self
            .tree
            .node(key)
            .ok_or(ProjectionError::MissingNode(key))?
            .style
            .clone();
        let mut projection = StyleProjection::from_style(current);
        let diagnostic = if property == "pointer-events" {
            match pointer_events(value) {
                Some(enabled) => {
                    self.tree.update_pointer_events(key, enabled)?;
                    None
                }
                None => Some(StyleDiagnostic::InvalidValue {
                    property: property.to_owned(),
                }),
            }
        } else if property == "user-select" {
            match text_selection(value) {
                Some(policy) => {
                    self.tree.update_text_selection(key, Some(policy))?;
                    None
                }
                None => Some(StyleDiagnostic::InvalidValue {
                    property: property.to_owned(),
                }),
            }
        } else if property == "z-index" {
            match z_index(value) {
                Some(z_index) => {
                    self.tree.update_z_index(key, z_index)?;
                    None
                }
                None => Some(StyleDiagnostic::InvalidValue {
                    property: property.to_owned(),
                }),
            }
        } else {
            project_ir(&mut projection, property, value)
        };
        self.update_style(key, projection.into_style())?;
        Ok(diagnostic)
    }

    pub fn set_stylesheet(
        &mut self,
        stylesheet: wabou_style::stylesheet::StyleSheet,
    ) -> Result<(), String> {
        stylesheet.validate().map_err(str::to_owned)?;
        let selected = stylesheet.color_themes.as_ref().and_then(|themes| {
            let name = self
                .active_color_theme
                .as_ref()
                .filter(|name| themes.themes.contains_key(*name))
                .unwrap_or(&themes.default);
            themes
                .themes
                .get(name)
                .map(|theme| (name.clone(), theme.colors.clone()))
        });
        self.active_color_theme = selected.as_ref().map(|(name, _)| name.clone());
        self.active_theme_colors = selected.map(|(_, colors)| colors);
        self.stylesheet = Some(stylesheet);
        self.resolved_class_styles.clear();
        self.recompute_all_styles()
    }

    pub fn set_color_theme(&mut self, name: &str) -> Result<bool, String> {
        let colors = self
            .stylesheet
            .as_ref()
            .and_then(|sheet| sheet.color_themes.as_ref())
            .and_then(|themes| themes.themes.get(name))
            .map(|theme| theme.colors.clone())
            .ok_or_else(|| format!("unknown Wabou color theme `{name}`"))?;
        if self.active_color_theme.as_deref() == Some(name)
            && self.active_theme_colors.as_ref() == Some(&colors)
        {
            return Ok(false);
        }
        self.active_color_theme = Some(name.to_owned());
        self.active_theme_colors = Some(colors);
        self.resolved_class_styles.clear();
        self.recompute_all_styles()?;
        Ok(true)
    }

    pub fn set_color_palette(
        &mut self,
        colors: std::collections::HashMap<String, u32>,
    ) -> Result<bool, String> {
        if self.active_theme_colors.as_ref() == Some(&colors) {
            return Ok(false);
        }
        self.active_theme_colors = Some(colors);
        self.resolved_class_styles.clear();
        self.recompute_all_styles()?;
        Ok(true)
    }

    pub fn set_ordered_color_palette(&mut self, colors: Vec<u32>) -> Result<bool, String> {
        let mut tokens = self
            .stylesheet
            .as_ref()
            .and_then(|sheet| sheet.color_themes.as_ref())
            .and_then(|themes| themes.themes.get(&themes.default))
            .map(|theme| theme.colors.keys().cloned().collect::<Vec<_>>())
            .ok_or_else(|| "stylesheet does not declare color theme tokens".to_owned())?;
        tokens.sort_unstable();
        if tokens.len() != colors.len() {
            return Err(format!(
                "color palette has {} values but stylesheet declares {} tokens",
                colors.len(),
                tokens.len()
            ));
        }
        self.set_color_palette(tokens.into_iter().zip(colors).collect())
    }

    pub fn active_theme_snapshot(&self) -> Option<GpuiThemeSnapshot> {
        let themes = self.stylesheet.as_ref()?.color_themes.as_ref()?;
        let name = self.active_color_theme.as_ref()?;
        let theme = themes.themes.get(name)?;
        Some(GpuiThemeSnapshot {
            dark: theme._appearance == wabou_style::stylesheet::Appearance::Dark,
            colors: self.active_theme_colors.clone()?,
        })
    }

    fn recompute_all_styles(&mut self) -> Result<(), String> {
        let keys = self
            .tree
            .keys()
            .filter(|key| *key != NodeKey::ROOT)
            .collect::<Vec<_>>();
        for key in keys {
            self.recompute_style(key)
                .map_err(|error| format!("cannot recompute {key}: {error:?}"))?;
        }
        Ok(())
    }

    fn recompute_style(&mut self, key: NodeKey) -> Result<(), ProjectionError> {
        #[cfg(test)]
        {
            self.style_recomputation_count = self.style_recomputation_count.saturating_add(1);
        }
        let mut projection = StyleProjection::default();
        let mut pointer_events_enabled = true;
        let mut text_selection_policy = None;
        let mut z_index_value = 0;
        let mut diagnostics = Vec::new();
        if let Some(stylesheet) = &self.stylesheet {
            let classes = self.classes.get(&key).cloned().unwrap_or_default();
            let resolved = if let Some(resolved) = self.resolved_class_styles.get(&classes) {
                resolved.clone()
            } else {
                let resolved = wabou_style::stylesheet::resolve_classes_with_colors(
                    stylesheet,
                    &classes.iter().map(String::as_str).collect::<Vec<_>>(),
                    self.active_theme_colors.as_ref(),
                );
                self.resolved_class_styles.insert(classes, resolved.clone());
                resolved
            };
            diagnostics.extend(resolved.diagnostics);
            for declaration in resolved.declarations {
                let diagnostic = if declaration.property == "pointer-events" {
                    match pointer_events(&declaration.value) {
                        Some(enabled) => {
                            pointer_events_enabled = enabled;
                            None
                        }
                        None => Some(StyleDiagnostic::InvalidValue {
                            property: declaration.property.clone(),
                        }),
                    }
                } else if declaration.property == "user-select" {
                    match text_selection(&declaration.value) {
                        Some(policy) => {
                            text_selection_policy = Some(policy);
                            None
                        }
                        None => Some(StyleDiagnostic::InvalidValue {
                            property: declaration.property.clone(),
                        }),
                    }
                } else if declaration.property == "z-index" {
                    match z_index(&declaration.value) {
                        Some(value) => {
                            z_index_value = value;
                            None
                        }
                        None => Some(StyleDiagnostic::InvalidValue {
                            property: declaration.property.clone(),
                        }),
                    }
                } else {
                    project_ir(&mut projection, &declaration.property, &declaration.value)
                };
                if let Some(diagnostic) = diagnostic {
                    diagnostics.push(format!("{diagnostic:?}"));
                }
            }
        }
        if let Some(styles) = self.inline_styles.get(&key) {
            for (property, value) in styles {
                let diagnostic = if property == "pointer-events" {
                    match pointer_events(value) {
                        Some(enabled) => {
                            pointer_events_enabled = enabled;
                            None
                        }
                        None => Some(StyleDiagnostic::InvalidValue {
                            property: property.clone(),
                        }),
                    }
                } else if property == "user-select" {
                    match text_selection(value) {
                        Some(policy) => {
                            text_selection_policy = Some(policy);
                            None
                        }
                        None => Some(StyleDiagnostic::InvalidValue {
                            property: property.clone(),
                        }),
                    }
                } else if property == "z-index" {
                    match z_index(value) {
                        Some(value) => {
                            z_index_value = value;
                            None
                        }
                        None => Some(StyleDiagnostic::InvalidValue {
                            property: property.clone(),
                        }),
                    }
                } else {
                    project_ir(&mut projection, property, value)
                };
                if let Some(diagnostic) = diagnostic {
                    diagnostics.push(format!("{diagnostic:?}"));
                }
            }
        }
        self.tree
            .update_pointer_events(key, pointer_events_enabled)?;
        self.tree
            .update_text_selection(key, text_selection_policy)?;
        self.tree.update_z_index(key, z_index_value)?;
        if diagnostics.is_empty() {
            self.style_diagnostics.remove(&key);
        } else {
            self.style_diagnostics.insert(key, diagnostics);
        }
        let mut style = projection.into_style();
        if let Some(node) = self.tree.node(key) {
            style.text.line_clamp = usize::try_from(node.text_max_lines)
                .ok()
                .filter(|lines| *lines > 0);
            if node.text_behavior & TEXT_BEHAVIOR_SINGLE_LINE != 0 {
                style.text.white_space = Some(crate::gpui::WhiteSpace::Nowrap);
            }
        }
        self.update_style(key, style)
    }

    #[cfg(test)]
    pub(crate) fn tree(&self) -> &ProjectionTree {
        &self.tree
    }

    #[doc(hidden)]
    pub fn style(&self, key: NodeKey) -> Option<&crate::gpui::Style> {
        self.tree.node(key).map(|node| &node.style)
    }

    pub fn style_diagnostics(&self, key: NodeKey) -> &[String] {
        self.style_diagnostics.get(&key).map_or(&[], Vec::as_slice)
    }

    #[cfg(test)]
    fn style_recomputation_count(&self) -> usize {
        self.style_recomputation_count
    }
}

fn projected_style_dirty_kind(
    previous: &crate::gpui::Style,
    next: &crate::gpui::Style,
) -> DirtyKind {
    let text_layout_changed = previous.text.font_family != next.text.font_family
        || previous.text.font_features != next.text.font_features
        || previous.text.font_fallbacks != next.text.font_fallbacks
        || previous.text.font_size != next.text.font_size
        || previous.text.line_height != next.text.line_height
        || previous.text.font_weight != next.text.font_weight
        || previous.text.font_style != next.text.font_style
        || previous.text.white_space != next.text.white_space
        || previous.text.text_overflow != next.text.text_overflow
        || previous.text.text_align != next.text.text_align
        || previous.text.line_clamp != next.text.line_clamp
        || previous.text.letter_spacing != next.text.letter_spacing
        || previous.text.text_transform != next.text.text_transform;
    let layout_changed = previous.display != next.display
        || previous.overflow != next.overflow
        || previous.scrollbar_width != next.scrollbar_width
        || previous.allow_concurrent_scroll != next.allow_concurrent_scroll
        || previous.restrict_scroll_to_axis != next.restrict_scroll_to_axis
        || previous.position != next.position
        || previous.inset != next.inset
        || previous.size != next.size
        || previous.min_size != next.min_size
        || previous.max_size != next.max_size
        || previous.aspect_ratio != next.aspect_ratio
        || previous.margin != next.margin
        || previous.padding != next.padding
        || previous.border_widths != next.border_widths
        || previous.align_items != next.align_items
        || previous.align_self != next.align_self
        || previous.align_content != next.align_content
        || previous.justify_content != next.justify_content
        || previous.gap != next.gap
        || previous.flex_direction != next.flex_direction
        || previous.flex_wrap != next.flex_wrap
        || previous.flex_basis != next.flex_basis
        || previous.flex_grow != next.flex_grow
        || previous.flex_shrink != next.flex_shrink
        || previous.grid_cols != next.grid_cols
        || previous.grid_rows != next.grid_rows
        || previous.grid_location != next.grid_location
        || text_layout_changed;
    let paint_changed = previous.visibility != next.visibility
        || previous.background != next.background
        || previous.border_color != next.border_color
        || previous.border_style != next.border_style
        || previous.corner_radii != next.corner_radii
        || previous.box_shadow != next.box_shadow
        || previous.filter != next.filter
        || previous.backdrop_filter != next.backdrop_filter
        || previous.opacity != next.opacity
        || previous.text.color != next.text.color
        || previous.text.background_color != next.text.background_color
        || previous.text.underline != next.text.underline
        || previous.text.strikethrough != next.text.strikethrough
        || layout_changed;
    #[cfg(debug_assertions)]
    let paint_changed =
        paint_changed || previous.debug != next.debug || previous.debug_below != next.debug_below;
    let interaction_changed = previous.mouse_cursor != next.mouse_cursor
        || previous.overflow != next.overflow
        || previous.allow_concurrent_scroll != next.allow_concurrent_scroll
        || previous.restrict_scroll_to_axis != next.restrict_scroll_to_axis;

    let mut dirty = DirtyKind::empty();
    if layout_changed {
        dirty |= DirtyKind::LAYOUT;
    }
    if text_layout_changed {
        dirty |= DirtyKind::TEXT;
    }
    if paint_changed {
        dirty |= DirtyKind::PAINT;
    }
    if interaction_changed {
        dirty |= DirtyKind::INTERACTION;
    }
    dirty
}

fn pointer_events(value: &IrValue) -> Option<bool> {
    let IrValue::Keyword { value } = value else {
        return None;
    };
    match value.as_str() {
        "auto" => Some(true),
        "none" => Some(false),
        _ => None,
    }
}

fn text_selection(value: &IrValue) -> Option<TextSelectionPolicy> {
    let IrValue::Keyword { value } = value else {
        return None;
    };
    match value.as_str() {
        "text" => Some(TextSelectionPolicy::Text),
        "none" => Some(TextSelectionPolicy::None),
        "all" => Some(TextSelectionPolicy::All),
        _ => None,
    }
}

fn z_index(value: &IrValue) -> Option<usize> {
    let IrValue::Number { value } = value else {
        return None;
    };
    (value.is_finite() && *value >= 0.0 && value.fract() == 0.0 && *value <= usize::MAX as f32)
        .then_some(*value as usize)
}

fn is_translation_matrix(matrix: [f32; 6]) -> bool {
    matrix[0] == 1.0 && matrix[1] == 0.0 && matrix[2] == 0.0 && matrix[3] == 1.0
}

fn protocol_style_value(value: StyleValue) -> IrValue {
    match value {
        StyleValue::Px(value) => IrValue::Length {
            value: IrLength::Px { value },
        },
        StyleValue::Percent(value) => IrValue::Length {
            value: IrLength::Percent { value },
        },
        StyleValue::Number(value) => IrValue::Number { value },
        StyleValue::Boolean(value) => IrValue::Boolean { value },
        StyleValue::Color(rgba) => IrValue::Color {
            value: IrColor::Literal { rgba },
        },
        StyleValue::Auto => IrValue::Length {
            value: IrLength::Auto,
        },
    }
}

fn protocol_shadow_value(shadow: ShadowValue) -> IrValue {
    let px = |value| IrValue::Length {
        value: IrLength::Px { value },
    };
    IrValue::Record {
        fields: std::collections::HashMap::from([
            ("x".to_owned(), px(shadow.offset_x)),
            ("y".to_owned(), px(shadow.offset_y)),
            ("spread".to_owned(), px(shadow.spread)),
            ("stdDev".to_owned(), px(shadow.std_dev)),
            (
                "color".to_owned(),
                IrValue::Color {
                    value: IrColor::Literal { rgba: shadow.color },
                },
            ),
        ]),
    }
}

pub fn project_ir(
    projection: &mut StyleProjection,
    property: &str,
    value: &IrValue,
) -> Option<StyleDiagnostic> {
    let value = tooling_value(value)?;
    projection
        .apply(&wabou_style::Declaration {
            property: property.to_owned(),
            value,
        })
        .err()
}

fn tooling_value(value: &IrValue) -> Option<wabou_style::Value> {
    Some(match value {
        IrValue::Keyword { value } => wabou_style::Value::Keyword {
            value: value.clone(),
        },
        IrValue::Boolean { value } => wabou_style::Value::Boolean { value: *value },
        IrValue::Number { value } => wabou_style::Value::Number { value: *value },
        IrValue::Length { value } => wabou_style::Value::Length {
            value: match value {
                IrLength::Px { value } => wabou_style::Length::Px { value: *value },
                IrLength::Percent { value } => wabou_style::Length::Percent { value: *value },
                IrLength::Auto => wabou_style::Length::Auto,
            },
        },
        IrValue::Color {
            value: IrColor::Literal { rgba },
        } => wabou_style::Value::Color {
            value: wabou_style::Color::Literal { rgba: *rgba },
        },
        IrValue::Color {
            value: IrColor::Token { .. },
        } => return None,
        IrValue::List { values } => wabou_style::Value::List {
            values: values.iter().map(tooling_value).collect::<Option<_>>()?,
        },
        IrValue::Record { fields } => wabou_style::Value::Record {
            fields: fields
                .iter()
                .map(|(key, value)| Some((key.clone(), tooling_value(value)?)))
                .collect::<Option<_>>()?,
        },
    })
}

fn gpui_style() -> crate::gpui::Style {
    crate::gpui::Style::default()
}

fn gpui_root_style() -> crate::gpui::Style {
    // The canonical host root is a bounded column, not an authored block.
    // A block root lets a `flex-1 min-h-0` page viewport take its intrinsic
    // content height, so scroll containers grow beyond the native window.
    crate::gpui::Style {
        display: crate::gpui::Display::Flex,
        flex_direction: crate::gpui::FlexDirection::Column,
        size: crate::gpui::size(
            crate::gpui::relative(1.0).into(),
            crate::gpui::relative(1.0).into(),
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpui::{DefiniteLength, Length};
    use image::ImageEncoder as _;

    fn key(lo: u32) -> NodeKey {
        NodeKey::new(lo, 1)
    }

    #[test]
    fn canonical_root_bounds_flexible_page_viewports_to_the_window() {
        let style = gpui_root_style();
        assert_eq!(style.display, crate::gpui::Display::Flex);
        assert_eq!(style.flex_direction, crate::gpui::FlexDirection::Column);
        assert_eq!(
            style.size,
            crate::gpui::size(
                crate::gpui::relative(1.0).into(),
                crate::gpui::relative(1.0).into(),
            )
        );
    }

    #[test]
    fn completed_solid_frame_projects_structure_and_text_with_protocol_identity() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::CreateText {
                            id: key(3),
                            text: "hello GPUI",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                        Op::SetText {
                            id: key(3),
                            text: "updated once per flush",
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert!(projection.finish_frame());

        let tree = projection.tree();
        assert_eq!(tree.node(NodeKey::ROOT).unwrap().children, [key(2)]);
        assert_eq!(tree.node(key(2)).unwrap().children, [key(3)]);
        assert_eq!(
            tree.node(key(2)).unwrap().kind,
            ProjectedNodeKind::Element("view".into())
        );
        assert_eq!(
            tree.node(key(3)).unwrap().text.as_deref(),
            Some("updated once per flush")
        );
    }

    #[test]
    fn completed_frames_advance_only_the_nearest_projection_boundary() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::CreateText {
                            id: key(3),
                            text: "first",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                        Op::SetProjectionBoundary {
                            id: key(2),
                            enabled: true,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.finish_frame());
        let initial_changes = projection.take_boundary_changes();
        assert!(initial_changes.contains_key(&NodeKey::ROOT));
        assert!(initial_changes.contains_key(&key(2)));
        let initial = projection.projection_boundary_revisions().clone();

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::SetText {
                        id: key(3),
                        text: "second",
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.finish_frame());
        let changes = projection.take_boundary_changes();
        let updated = projection.projection_boundary_revisions().clone();

        assert_eq!(
            changes.keys().copied().collect::<Vec<_>>(),
            vec![NodeKey::ROOT, key(2)],
            "the runtime receives exact changed owners rather than preparing every boundary"
        );
        assert_eq!(changes[&NodeKey::ROOT], DirtyKind::LAYOUT);
        assert!(changes[&key(2)].contains(DirtyKind::TEXT));
        assert!(projection.take_boundary_changes().is_empty());

        assert_eq!(
            updated[&NodeKey::ROOT].structure,
            initial[&NodeKey::ROOT].structure,
            "descendant text must not rebuild parent structure"
        );
        assert!(
            updated[&NodeKey::ROOT].layout > initial[&NodeKey::ROOT].layout,
            "a boundary's intrinsic size may change, so its parent placement must relayout"
        );
        assert_eq!(updated[&key(2)].structure, initial[&key(2)].structure);
        assert!(updated[&key(2)].layout > initial[&key(2)].layout);
        assert!(updated[&key(2)].paint > initial[&key(2)].paint);
    }

    #[test]
    fn boundary_root_layout_changes_invalidate_its_parent_placement_only() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let width = atoms.intern("width");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(3),
                        },
                        Op::SetProjectionBoundary {
                            id: key(2),
                            enabled: true,
                        },
                        Op::SetProjectionBoundary {
                            id: key(3),
                            enabled: true,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.finish_frame());
        projection.take_boundary_changes();
        let initial = projection.projection_boundary_revisions().clone();

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::SetStyleValue {
                        id: key(2),
                        prop: width,
                        value: StyleValue::Px(480.0),
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.finish_frame());
        let changes = projection.take_boundary_changes();
        let updated = projection.projection_boundary_revisions();

        assert_eq!(
            changes.keys().copied().collect::<Vec<_>>(),
            vec![NodeKey::ROOT, key(2)],
            "the unchanged sibling boundary must not reach GPUI synchronization"
        );
        assert!(updated[&key(2)].layout > initial[&key(2)].layout);
        assert_eq!(updated[&key(2)].structure, initial[&key(2)].structure);
        assert_eq!(updated[&key(3)], initial[&key(3)]);
        assert!(updated[&NodeKey::ROOT].layout > initial[&NodeKey::ROOT].layout);
        assert_eq!(
            updated[&NodeKey::ROOT].structure,
            initial[&NodeKey::ROOT].structure
        );
    }

    #[test]
    fn projection_boundary_index_tracks_recursive_removal() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                        Op::SetProjectionBoundary {
                            id: key(2),
                            enabled: true,
                        },
                        Op::SetProjectionBoundary {
                            id: key(3),
                            enabled: true,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.finish_frame());
        assert_eq!(
            projection.render_snapshot().projection_boundaries(),
            [NodeKey::ROOT, key(2), key(3)]
        );

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::DropNode { id: key(2) }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.finish_frame());
        assert_eq!(
            projection.render_snapshot().projection_boundaries(),
            [NodeKey::ROOT]
        );
        assert_eq!(
            projection
                .projection_boundary_revisions()
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            [NodeKey::ROOT]
        );
    }

    #[test]
    fn event_and_focus_contracts_are_retained_without_legacy_document_state() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let button = atoms.intern("button");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: button,
                        },
                        Op::AddEventListener {
                            id: key(2),
                            event_type: wabou_protocol::event::CLICK,
                        },
                        Op::SetInteractionPolicy {
                            id: key(2),
                            flags: wabou_protocol::INTERACTION_POLICY_FOCUSABLE
                                | wabou_protocol::INTERACTION_POLICY_CONTAIN_FOCUS,
                            focus_order: 7,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let node = projection.tree().node(key(2)).unwrap();
        assert_eq!(node.listeners, [wabou_protocol::event::CLICK].into());
        assert_eq!(node.focus_order, Some(7));
        assert!(node.focus_contained);
        assert!(!node.interaction_blocked);
        assert!(projection.has_listener_in_chain(key(2), wabou_protocol::event::CLICK));

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::RemoveEventListener {
                        id: key(2),
                        event_type: wabou_protocol::event::CLICK,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.tree().node(key(2)).unwrap().listeners.is_empty());
        assert!(!projection.has_listener_in_chain(key(2), wabou_protocol::event::CLICK));
    }

    #[test]
    fn pointer_event_policy_is_projected_as_gpui_node_metadata() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![Op::CreateElement {
                        id: key(2),
                        tag: view,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert_eq!(
            projection
                .apply_style_declaration(
                    key(2),
                    "pointer-events",
                    &IrValue::Keyword {
                        value: "none".to_owned(),
                    },
                )
                .unwrap(),
            None
        );
        assert!(!projection.tree().node(key(2)).unwrap().pointer_events);
        assert_eq!(
            projection
                .apply_style_declaration(
                    key(2),
                    "pointer-events",
                    &IrValue::Keyword {
                        value: "auto".to_owned(),
                    },
                )
                .unwrap(),
            None
        );
        assert!(projection.tree().node(key(2)).unwrap().pointer_events);

        assert_eq!(
            projection
                .apply_style_declaration(key(2), "z-index", &IrValue::Number { value: 42.0 },)
                .unwrap(),
            None
        );
        assert_eq!(projection.tree().node(key(2)).unwrap().z_index, 42);
    }

    #[test]
    fn user_select_policy_controls_native_text_participants_by_subtree() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::CreateText {
                            id: key(3),
                            text: "select me",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert!(
            projection.selectable_texts().is_empty(),
            "ordinary projected text is not selectable unless its subtree opts in"
        );
        projection
            .apply_style_declaration(
                key(2),
                "user-select",
                &IrValue::Keyword {
                    value: "text".to_owned(),
                },
            )
            .unwrap();
        assert_eq!(projection.selectable_texts()[0].key, key(3));

        projection
            .apply_style_declaration(
                key(3),
                "user-select",
                &IrValue::Keyword {
                    value: "all".to_owned(),
                },
            )
            .unwrap();
        let selectable = projection.selectable_texts();
        assert_eq!(selectable.len(), 1);
        assert!(selectable[0].select_all);
    }

    #[test]
    fn runtime_transform_is_retained_and_non_translation_is_diagnosed() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetTransform2D {
                            id: key(2),
                            matrix: [1.0, 0.0, 0.0, 1.0, 12.0, -4.0],
                        },
                        Op::SetOverlayPlane {
                            id: key(2),
                            plane: 2,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert_eq!(
            projection.tree().node(key(2)).unwrap().transform,
            [1.0, 0.0, 0.0, 1.0, 12.0, -4.0]
        );
        assert_eq!(projection.tree().node(key(2)).unwrap().overlay_plane, 2);
        assert_eq!(
            projection.tree().node(key(2)).unwrap().draw_priority(),
            2_000_000
        );

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::SetTransform2D {
                        id: key(2),
                        matrix: [0.5, 0.0, 0.0, 0.5, 0.0, 0.0],
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(!is_translation_matrix(
            projection.tree().node(key(2)).unwrap().transform
        ));
    }

    #[test]
    fn structure_and_style_publish_in_one_commit() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let initial_revision = projection.revision();
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        projection
            .apply_style_declaration(
                key(2),
                "background-color",
                &IrValue::Color {
                    value: IrColor::Literal { rgba: 0x4080_bfff },
                },
            )
            .unwrap();

        assert_eq!(projection.revision(), initial_revision);
        assert!(projection.finish_frame());
        assert_eq!(projection.revision(), initial_revision + 1);
        assert!(!projection.finish_frame());
        assert_eq!(projection.revision(), initial_revision + 1);
    }

    #[test]
    fn one_protocol_frame_resolves_each_nodes_final_style_once() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let panel = atoms.intern("panel");
        let width = atoms.intern("width");
        let opacity = atoms.intern("opacity");

        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetClassName {
                            id: key(2),
                            classes: vec![panel],
                        },
                        Op::SetStyleValue {
                            id: key(2),
                            prop: width,
                            value: StyleValue::Px(320.0),
                        },
                        Op::SetStyleValue {
                            id: key(2),
                            prop: opacity,
                            value: StyleValue::Number(0.5),
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert_eq!(projection.style_recomputation_count(), 1);
        let style = projection.style(key(2)).unwrap();
        assert_eq!(
            style.size.width,
            Length::Definite(DefiniteLength::Absolute(crate::gpui::px(320.0).into()))
        );
        assert_eq!(style.opacity, Some(0.5));
    }

    #[test]
    fn typed_inline_styles_apply_and_remove_without_the_legacy_applier() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let width = atoms.intern("width");
        let opacity = atoms.intern("opacity");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetStyleValue {
                            id: key(2),
                            prop: width,
                            value: StyleValue::Px(320.0),
                        },
                        Op::SetStyleValue {
                            id: key(2),
                            prop: opacity,
                            value: StyleValue::Number(0.625),
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert!(projection.finish_frame());
        let style = projection.style(key(2)).unwrap();
        assert_eq!(
            style.size.width,
            Length::Definite(DefiniteLength::Absolute(crate::gpui::px(320.0).into()))
        );
        assert_eq!(style.opacity, Some(0.625));

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::RemoveStyle {
                        id: key(2),
                        prop: width,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert!(projection.finish_frame());
        let style = projection.style(key(2)).unwrap();
        assert_eq!(style.size.width, Length::Auto);
        assert_eq!(style.opacity, Some(0.625));
    }

    #[test]
    fn string_inline_styles_share_the_neutral_cascade() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let width = atoms.intern("width");
        let background = atoms.intern("background-color");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetStyle {
                            id: key(2),
                            prop: width,
                            value: "20rem",
                        },
                        Op::SetStyle {
                            id: key(2),
                            prop: background,
                            value: "#123",
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let style = projection.style(key(2)).unwrap();
        assert_eq!(
            style.size.width,
            Length::Definite(DefiniteLength::Absolute(crate::gpui::px(320.0).into()))
        );
        assert_eq!(
            style.background,
            Some(crate::gpui::rgba(0x1122_33ff).into())
        );
    }

    #[test]
    fn dropped_nodes_do_not_retain_inline_style_state() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let width = atoms.intern("width");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetStyleValue {
                            id: key(2),
                            prop: width,
                            value: StyleValue::Px(320.0),
                        },
                        Op::DropNode { id: key(2) },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert!(!projection.contains(key(2)));

        let recreated = NodeKey::new(2, 2);
        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::CreateElement {
                        id: recreated,
                        tag: view,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert_eq!(
            projection.style(recreated).unwrap().size.width,
            Length::Auto
        );
    }

    #[test]
    fn subtree_drop_records_are_idempotent_for_each_released_js_handle() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                        // The Solid renderer releases every JS handle. Removing the
                        // parent already retires its retained subtree, so the child's
                        // release record must be harmless rather than corrupting the
                        // remainder of the frame.
                        Op::DropNode { id: key(2) },
                        Op::DropNode { id: key(3) },
                    ],
                },
                &atoms,
                |_| None,
            )
            .expect("releasing every handle in a removed subtree must be idempotent");

        assert!(!projection.contains(key(2)));
        assert!(!projection.contains(key(3)));
    }

    #[test]
    fn typed_shadow_batches_project_directly_to_gpui() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetShadows {
                            id: key(2),
                            shadows: vec![
                                ShadowValue {
                                    offset_x: 1.0,
                                    offset_y: 2.0,
                                    spread: 3.0,
                                    std_dev: 8.0,
                                    color: 0x1122_3380,
                                    radius: Some(12.0),
                                },
                                ShadowValue {
                                    offset_x: -2.0,
                                    offset_y: 4.0,
                                    spread: 0.0,
                                    std_dev: 6.0,
                                    color: 0x4455_66ff,
                                    radius: None,
                                },
                            ],
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let shadows = &projection.style(key(2)).unwrap().box_shadow;
        assert_eq!(shadows.len(), 2);
        assert_eq!(
            shadows[0].offset,
            crate::gpui::point(crate::gpui::px(1.0), crate::gpui::px(2.0))
        );
        assert_eq!(shadows[0].blur_radius, crate::gpui::px(8.0));
        assert_eq!(shadows[0].spread_radius, crate::gpui::px(3.0));
        assert_eq!(
            shadows[1].offset,
            crate::gpui::point(crate::gpui::px(-2.0), crate::gpui::px(4.0))
        );
    }

    #[test]
    fn stylesheet_classes_project_without_the_legacy_document() {
        use wabou_style::stylesheet::fixture::{declaration, px, rule, sheet};

        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let panel = atoms.intern("panel");
        let width = atoms.intern("width");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetClassName {
                            id: key(2),
                            classes: vec![panel],
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        projection
            .set_stylesheet(sheet(vec![rule(
                "panel",
                vec![declaration("width", px(320.0))],
            )]))
            .unwrap();

        assert_eq!(
            projection.style(key(2)).unwrap().size.width,
            Length::Definite(DefiniteLength::Absolute(crate::gpui::px(320.0).into()))
        );
        assert!(projection.style_diagnostics(key(2)).is_empty());

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::SetStyleValue {
                        id: key(2),
                        prop: width,
                        value: StyleValue::Px(480.0),
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert_eq!(
            projection.style(key(2)).unwrap().size.width,
            Length::Definite(DefiniteLength::Absolute(crate::gpui::px(480.0).into()))
        );
    }

    #[test]
    fn repeated_class_sets_share_resolution_and_refresh_after_theme_changes() {
        use wabou_style::stylesheet::{
            Appearance, ColorTheme, ColorThemes, StyleSheet,
            fixture::{color_token, declaration, rule},
        };

        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let swatch = atoms.intern("swatch");
        let mut ops = Vec::new();
        for index in 2..=401 {
            ops.push(Op::CreateElement {
                id: key(index),
                tag: view,
            });
            ops.push(Op::SetClassName {
                id: key(index),
                classes: vec![swatch],
            });
        }
        projection
            .apply_ops(&Frame { seq: 1, ops }, &atoms, |_| None)
            .unwrap();
        projection
            .set_stylesheet(
                StyleSheet::builder()
                    .color_themes(ColorThemes {
                        default: "light".into(),
                        themes: std::collections::HashMap::from([
                            (
                                "light".into(),
                                ColorTheme {
                                    _appearance: Appearance::Light,
                                    colors: std::collections::HashMap::from([(
                                        "surface".into(),
                                        0xffff_ffff,
                                    )]),
                                },
                            ),
                            (
                                "dark".into(),
                                ColorTheme {
                                    _appearance: Appearance::Dark,
                                    colors: std::collections::HashMap::from([(
                                        "surface".into(),
                                        0x1010_10ff,
                                    )]),
                                },
                            ),
                        ]),
                    })
                    .rules(vec![rule(
                        "swatch",
                        vec![declaration("background-color", color_token("surface"))],
                    )])
                    .build(),
            )
            .unwrap();

        assert_eq!(projection.resolved_class_styles.len(), 1);
        assert_eq!(
            projection.style(key(401)).unwrap().background,
            Some(crate::gpui::rgba(0xffff_ffff).into())
        );

        assert!(projection.set_color_theme("dark").unwrap());
        assert_eq!(projection.resolved_class_styles.len(), 1);
        assert_eq!(
            projection.style(key(2)).unwrap().background,
            Some(crate::gpui::rgba(0x1010_10ff).into())
        );
    }

    #[test]
    fn named_theme_switch_recomputes_gpui_classes_without_legacy_state() {
        use wabou_style::stylesheet::{
            Appearance, ColorTheme, ColorThemes, StyleSheet,
            fixture::{color_token, declaration, rule},
        };

        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let panel = atoms.intern("panel");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetClassName {
                            id: key(2),
                            classes: vec![panel],
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        projection
            .set_stylesheet(
                StyleSheet::builder()
                    .color_themes(ColorThemes {
                        default: "light".into(),
                        themes: std::collections::HashMap::from([
                            (
                                "light".into(),
                                ColorTheme {
                                    _appearance: Appearance::Light,
                                    colors: std::collections::HashMap::from([(
                                        "surface".into(),
                                        0xffff_ffff,
                                    )]),
                                },
                            ),
                            (
                                "dark".into(),
                                ColorTheme {
                                    _appearance: Appearance::Dark,
                                    colors: std::collections::HashMap::from([(
                                        "surface".into(),
                                        0x1010_10ff,
                                    )]),
                                },
                            ),
                        ]),
                    })
                    .rules(vec![rule(
                        "panel",
                        vec![declaration("background-color", color_token("surface"))],
                    )])
                    .build(),
            )
            .unwrap();

        assert_eq!(
            projection.style(key(2)).unwrap().background,
            Some(crate::gpui::rgba(0xffff_ffff).into())
        );
        assert_eq!(
            projection.active_theme_snapshot(),
            Some(GpuiThemeSnapshot {
                dark: false,
                colors: std::collections::HashMap::from([("surface".into(), 0xffff_ffff)]),
            })
        );
        assert!(projection.set_color_theme("dark").unwrap());
        assert_eq!(
            projection.style(key(2)).unwrap().background,
            Some(crate::gpui::rgba(0x1010_10ff).into())
        );
        assert_eq!(
            projection.active_theme_snapshot(),
            Some(GpuiThemeSnapshot {
                dark: true,
                colors: std::collections::HashMap::from([("surface".into(), 0x1010_10ff)]),
            })
        );
        assert!(!projection.set_color_theme("dark").unwrap());
    }

    #[test]
    fn named_theme_switch_updates_native_text_control_ink() {
        use wabou_style::stylesheet::{
            Appearance, ColorTheme, ColorThemes, StyleSheet,
            fixture::{color_token, declaration, rule},
        };

        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let input = atoms.intern("input");
        let field = atoms.intern("field");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: input,
                        },
                        Op::SetClassName {
                            id: key(2),
                            classes: vec![field],
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        projection
            .set_stylesheet(
                StyleSheet::builder()
                    .color_themes(ColorThemes {
                        default: "light".into(),
                        themes: std::collections::HashMap::from([
                            (
                                "light".into(),
                                ColorTheme {
                                    _appearance: Appearance::Light,
                                    colors: std::collections::HashMap::from([(
                                        "primary".into(),
                                        0x171a_1fff,
                                    )]),
                                },
                            ),
                            (
                                "dark".into(),
                                ColorTheme {
                                    _appearance: Appearance::Dark,
                                    colors: std::collections::HashMap::from([(
                                        "primary".into(),
                                        0xf2f4_f7ff,
                                    )]),
                                },
                            ),
                        ]),
                    })
                    .rules(vec![rule(
                        "field",
                        vec![declaration("color", color_token("primary"))],
                    )])
                    .build(),
            )
            .unwrap();

        let light = projection.text_controls()[0].style.foreground;
        assert_eq!(
            light,
            crate::gpui::rgb_to_hsla(crate::gpui::rgba(0x171a_1fff))
        );
        assert!(projection.set_color_theme("dark").unwrap());
        let dark = projection.text_controls()[0].style.foreground;
        assert_eq!(
            dark,
            crate::gpui::rgb_to_hsla(crate::gpui::rgba(0xf2f4_f7ff))
        );
        assert_ne!(dark, light);
    }

    #[test]
    fn graphic_sources_project_to_gpui_images_and_clear_explicitly() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let image_tag = atoms.intern("img");
        let svg_tag = atoms.intern("svg");
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&[1, 2, 3, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        let raster = std::sync::Arc::new(crate::gpui::Image::from_bytes(
            crate::gpui::ImageFormat::Png,
            png,
        ));
        let source = "image:1";

        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: image_tag,
                        },
                        Op::SetGraphicSource {
                            id: key(2),
                            kind: GRAPHIC_SOURCE_RESOURCE_RASTER,
                            source,
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: svg_tag,
                        },
                        Op::SetGraphicSource {
                            id: key(3),
                            kind: GRAPHIC_SOURCE_SVG,
                            source: r#"<svg viewBox="0 0 10 10"><circle cx="5" cy="5" r="4"/></svg>"#,
                        },
                        Op::SetTransform2D {
                            id: key(3),
                            matrix: [0.0, 1.0, -1.0, 0.0, 0.0, 0.0],
                        },
                    ],
                },
                &atoms,
                |_| Some(raster.clone()),
            )
            .unwrap();

        assert_eq!(
            projection
                .tree()
                .node(key(2))
                .unwrap()
                .image
                .as_ref()
                .unwrap()
                .format(),
            crate::gpui::ImageFormat::Png
        );
        assert!(projection.tree().node(key(3)).unwrap().image.is_none());
        assert!(projection.tree().node(key(3)).unwrap().svg_source.is_some());
        projection.layout_bounds.borrow_mut().insert(
            key(3),
            crate::gpui::Bounds::new(
                crate::gpui::point(crate::gpui::px(0.0), crate::gpui::px(0.0)),
                crate::gpui::size(crate::gpui::px(10.0), crate::gpui::px(10.0)),
            ),
        );
        assert!(
            projection
                .layout_snapshot()
                .into_iter()
                .find(|node| node.key == key(3))
                .unwrap()
                .style_diagnostics
                .is_empty(),
            "GPUI paints the complete affine matrix for retained inline SVG"
        );
        projection.graphic_paint_states.borrow_mut().insert(
            key(3),
            crate::element::ProjectedGraphicPaintState::SvgPainted {
                bounds: crate::gpui::Bounds::new(
                    crate::gpui::point(crate::gpui::px(0.0), crate::gpui::px(0.0)),
                    crate::gpui::size(crate::gpui::px(10.0), crate::gpui::px(10.0)),
                ),
                color: crate::gpui::transparent_black(),
            },
        );
        assert_eq!(
            projection
                .layout_snapshot()
                .into_iter()
                .find(|node| node.key == key(3))
                .unwrap()
                .style_diagnostics,
            &["GPUI painted SVG with a fully transparent inherited color"]
        );

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![
                        Op::ClearGraphicSource {
                            id: key(2),
                            kind: GRAPHIC_SOURCE_RESOURCE_RASTER,
                        },
                        Op::ClearGraphicSource {
                            id: key(3),
                            kind: GRAPHIC_SOURCE_SVG,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.tree().node(key(2)).unwrap().image.is_none());
        assert!(projection.tree().node(key(3)).unwrap().svg_source.is_none());
    }

    #[test]
    fn svg_layout_diagnostics_distinguish_missing_source_from_zero_paint_bounds() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let svg_tag = atoms.intern("svg");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: svg_tag,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let diagnostics = &projection.layout_snapshot()[1].style_diagnostics;
        assert_eq!(diagnostics, &["GPUI SVG node has no projected source"]);

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::SetGraphicSource {
                        id: key(2),
                        kind: GRAPHIC_SOURCE_SVG,
                        source: r#"<svg viewBox="0 0 10 10"><circle cx="5" cy="5" r="4"/></svg>"#,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let diagnostics = &projection.layout_snapshot()[1].style_diagnostics;
        assert_eq!(
            diagnostics,
            &["GPUI SVG source cannot paint into 0.0x0.0 bounds"]
        );
    }

    #[test]
    fn text_control_descriptors_follow_attached_generational_nodes_and_authored_state() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let input = atoms.intern("input");
        let textarea = atoms.intern("textarea");
        let editor = atoms.intern("editor");
        let value = atoms.intern("value");
        let placeholder = atoms.intern("placeholder");
        let disabled = atoms.intern("disabled");
        let submit_on_enter = atoms.intern("submitOnEnter");
        let language = atoms.intern("language");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: input,
                        },
                        Op::SetAttribute {
                            id: key(2),
                            name: value,
                            value: "typed",
                        },
                        Op::SetAttribute {
                            id: key(2),
                            name: placeholder,
                            value: "Search",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: textarea,
                        },
                        Op::SetAttribute {
                            id: key(3),
                            name: disabled,
                            value: "",
                        },
                        Op::SetAttribute {
                            id: key(3),
                            name: submit_on_enter,
                            value: "",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(3),
                        },
                        Op::CreateElement {
                            id: key(4),
                            tag: editor,
                        },
                        Op::SetAttribute {
                            id: key(4),
                            name: language,
                            value: "json",
                        },
                        Op::SetAttribute {
                            id: key(4),
                            name: value,
                            value: "{\"enabled\":true}",
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(4),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert_eq!(
            projection.text_controls(),
            vec![
                GpuiTextControl {
                    key: key(2),
                    kind: GpuiTextControlKind::Input,
                    language: None,
                    value: "typed".into(),
                    placeholder: "Search".into(),
                    disabled: false,
                    readonly: false,
                    submit_on_enter: false,
                    authored_value_revision: 1,
                    native_edit_revision: 0,
                    native_ack_revision: 0,
                    style: GpuiTextControlStyle::default(),
                },
                GpuiTextControl {
                    key: key(3),
                    kind: GpuiTextControlKind::Textarea,
                    language: None,
                    value: String::new(),
                    placeholder: String::new(),
                    disabled: true,
                    readonly: false,
                    submit_on_enter: true,
                    authored_value_revision: 0,
                    native_edit_revision: 0,
                    native_ack_revision: 0,
                    style: GpuiTextControlStyle::default(),
                },
                GpuiTextControl {
                    key: key(4),
                    kind: GpuiTextControlKind::Editor,
                    language: Some("json".into()),
                    value: "{\"enabled\":true}".into(),
                    placeholder: String::new(),
                    disabled: false,
                    readonly: false,
                    submit_on_enter: false,
                    authored_value_revision: 2,
                    native_edit_revision: 0,
                    native_ack_revision: 0,
                    style: GpuiTextControlStyle::default(),
                },
            ]
        );

        projection
            .update_native_text_value(key(2), "typed natively", 7)
            .unwrap();
        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::AcknowledgeTextValue {
                        id: key(2),
                        revision: 7,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        let input = projection
            .text_controls()
            .into_iter()
            .find(|control| control.key == key(2))
            .unwrap();
        assert_eq!(input.value, "typed natively");
        assert_eq!(input.authored_value_revision, 1);
        assert_eq!(input.native_edit_revision, 7);
        assert_eq!(input.native_ack_revision, 7);

        projection
            .apply_ops(
                &Frame {
                    seq: 3,
                    ops: vec![Op::SetAttribute {
                        id: key(2),
                        name: value,
                        value: "normalized by JavaScript",
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        let input = projection
            .text_controls()
            .into_iter()
            .find(|control| control.key == key(2))
            .unwrap();
        assert_eq!(input.value, "normalized by JavaScript");
        assert!(input.authored_value_revision > 1);
        assert_eq!(input.native_edit_revision, 7);
        assert_eq!(input.native_ack_revision, 7);

        projection
            .apply_ops(
                &Frame {
                    seq: 4,
                    ops: vec![Op::RemoveChild {
                        parent: NodeKey::ROOT,
                        child: key(2),
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert_eq!(
            projection
                .text_controls()
                .into_iter()
                .map(|control| control.key)
                .collect::<Vec<_>>(),
            [key(3), key(4)]
        );
    }

    #[test]
    fn native_widget_descriptors_preserve_authored_state_and_generational_identity() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let fractal = atoms.intern("fractal");
        let view = atoms.intern("view");
        let center_x = atoms.intern("center-x");
        let recreated = NodeKey::new(2, 7);

        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: recreated,
                            tag: fractal,
                        },
                        Op::SetAttribute {
                            id: recreated,
                            name: center_x,
                            value: "-0.745",
                        },
                        Op::SetWidgetConfig {
                            id: recreated,
                            json: r#"{"iterations":96}"#,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: recreated,
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(3),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let widgets = projection.native_widgets(|tag| tag == "fractal");
        assert_eq!(widgets.len(), 1);
        assert_eq!(widgets[0].key, recreated);
        assert_eq!(widgets[0].tag.as_ref(), "fractal");
        assert_eq!(
            widgets[0].attributes.get("center-x").map(AsRef::as_ref),
            Some("-0.745")
        );
        assert_eq!(widgets[0].config.as_deref(), Some(r#"{"iterations":96}"#));

        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::RemoveWidgetConfig { id: recreated }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert_eq!(
            projection.native_widgets(|tag| tag == "fractal")[0].config,
            None
        );

        projection
            .apply_ops(
                &Frame {
                    seq: 3,
                    ops: vec![Op::RemoveChild {
                        parent: NodeKey::ROOT,
                        child: recreated,
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        assert!(projection.native_widgets(|tag| tag == "fractal").is_empty());
    }

    #[test]
    fn layout_snapshot_marks_descendants_of_detached_roots_unreachable() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        projection
            .apply_ops(
                &Frame {
                    seq: 2,
                    ops: vec![Op::RemoveChild {
                        parent: NodeKey::ROOT,
                        child: key(2),
                    }],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let snapshot = projection.layout_snapshot();
        assert!(
            !snapshot
                .iter()
                .find(|node| node.key == key(2))
                .unwrap()
                .attached
        );
        assert!(
            !snapshot
                .iter()
                .find(|node| node.key == key(3))
                .unwrap()
                .attached,
            "a locally attached child is not rendered when its ancestor is detached"
        );
    }

    #[test]
    fn layout_snapshot_excludes_display_none_subtrees_even_with_stale_bounds() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        let bounds = crate::gpui::Bounds::new(
            crate::gpui::point(crate::gpui::px(0.0), crate::gpui::px(0.0)),
            crate::gpui::size(crate::gpui::px(100.0), crate::gpui::px(40.0)),
        );
        projection
            .layout_bounds
            .borrow_mut()
            .extend([(key(2), bounds), (key(3), bounds)]);
        let hidden = crate::gpui::Style {
            display: crate::gpui::Display::None,
            ..Default::default()
        };
        projection.update_style(key(2), hidden).unwrap();

        let snapshot = projection.layout_snapshot();
        assert!(
            !snapshot
                .iter()
                .find(|node| node.key == key(2))
                .unwrap()
                .attached
        );
        assert!(
            !snapshot
                .iter()
                .find(|node| node.key == key(3))
                .unwrap()
                .attached,
            "a child with stale bounds is not exposed below display:none"
        );
    }

    #[test]
    fn text_protocol_projects_gpui_native_wrapping_and_line_clamp() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let text = atoms.intern("text");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: text,
                        },
                        Op::SetTextBehavior {
                            id: key(2),
                            flags: wabou_protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT
                                | wabou_protocol::TEXT_BEHAVIOR_SINGLE_LINE,
                        },
                        Op::SetTextMaxLines {
                            id: key(2),
                            max_lines: 1,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let node = projection.tree().node(key(2)).unwrap();
        assert_eq!(
            node.text_behavior,
            wabou_protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT
                | wabou_protocol::TEXT_BEHAVIOR_SINGLE_LINE
        );
        assert_eq!(node.text_max_lines, 1);
        assert_eq!(node.style.text.line_clamp, Some(1));
        assert_eq!(
            node.style.text.white_space,
            Some(crate::gpui::WhiteSpace::Nowrap)
        );
    }

    #[test]
    fn styled_text_snapshot_preserves_one_native_paragraph() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let text = atoms.intern("text");
        let span = atoms.intern("text-span");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: text,
                        },
                        Op::SetTextBehavior {
                            id: key(2),
                            flags: wabou_protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT
                                | wabou_protocol::TEXT_BEHAVIOR_AGGREGATE_STYLED,
                        },
                        Op::CreateText {
                            id: key(3),
                            text: "Before ",
                        },
                        Op::CreateElement {
                            id: key(4),
                            tag: span,
                        },
                        Op::CreateText {
                            id: key(5),
                            text: "code",
                        },
                        Op::CreateText {
                            id: key(6),
                            text: " after.",
                        },
                        Op::AppendChild {
                            parent: key(4),
                            child: key(5),
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(4),
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(6),
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        let tree = projection.tree().snapshot();
        assert_eq!(
            crate::element::projected_text(&tree, tree.node(key(2)).unwrap()).as_deref(),
            Some("Before code after.")
        );
        projection.layout_bounds.borrow_mut().extend([
            (
                NodeKey::ROOT,
                crate::gpui::Bounds::new(
                    crate::gpui::point(crate::gpui::px(0.0), crate::gpui::px(0.0)),
                    crate::gpui::size(crate::gpui::px(320.0), crate::gpui::px(120.0)),
                ),
            ),
            (
                key(2),
                crate::gpui::Bounds::new(
                    crate::gpui::point(crate::gpui::px(16.0), crate::gpui::px(16.0)),
                    crate::gpui::size(crate::gpui::px(288.0), crate::gpui::px(26.0)),
                ),
            ),
        ]);
        let layout = projection.layout_snapshot();
        let paragraph = layout.iter().find(|node| node.key == key(2)).unwrap();
        assert!(paragraph.attached);
        assert_eq!(paragraph.computed.font_size, Some(16.0));
        assert_eq!(paragraph.computed.font_weight, Some(400.0));
        assert!(
            !layout
                .iter()
                .find(|node| node.key == key(4))
                .unwrap()
                .attached,
            "a text span absorbed into GPUI StyledText is retained but not independently rendered"
        );
    }

    #[test]
    fn imperative_ops_are_typed_commands_and_unmigrated_ops_are_observable() {
        let mut projection = GpuiProjection::new();
        let atoms = AtomPool::default();
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::FocusNode { id: key(2) },
                        Op::BlurNode { id: key(2) },
                        Op::ScrollTo {
                            id: key(3),
                            x: 4.0,
                            y: 8.0,
                        },
                        Op::ScrollBy {
                            id: key(3),
                            x: -1.0,
                            y: 2.0,
                        },
                        Op::SetTextSelection {
                            id: key(4),
                            anchor: 2,
                            head: 7,
                        },
                        Op::TextCommand {
                            id: key(4),
                            command: 1,
                        },
                        Op::TextCommand {
                            id: key(4),
                            command: 2,
                        },
                        Op::TextCommand {
                            id: key(4),
                            command: 3,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert_eq!(
            projection.take_commands(),
            vec![
                GpuiCommand::Focus { id: key(2) },
                GpuiCommand::Blur { id: key(2) },
                GpuiCommand::ScrollTo {
                    id: key(3),
                    x: 4.0,
                    y: 8.0,
                },
                GpuiCommand::ScrollBy {
                    id: key(3),
                    x: -1.0,
                    y: 2.0,
                },
                GpuiCommand::SetTextSelection {
                    id: key(4),
                    anchor: 2,
                    head: 7,
                },
                GpuiCommand::Text {
                    id: key(4),
                    command: GpuiTextCommand::SelectAll,
                },
                GpuiCommand::Text {
                    id: key(4),
                    command: GpuiTextCommand::Undo,
                },
                GpuiCommand::Text {
                    id: key(4),
                    command: GpuiTextCommand::Redo,
                },
            ]
        );
        assert!(projection.protocol_gaps().is_empty());
    }

    #[test]
    fn synthetic_wheel_scrolls_the_nearest_projected_scroll_ancestor() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        let overflow_y = atoms.intern("overflow-y");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetStyle {
                            id: key(2),
                            prop: overflow_y,
                            value: "scroll",
                        },
                        Op::CreateElement {
                            id: key(3),
                            tag: view,
                        },
                        Op::AppendChild {
                            parent: key(2),
                            child: key(3),
                        },
                        Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: key(2),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert_eq!(
            projection.apply_wheel_delta(key(3), 0.0, 80.0),
            Some(crate::ProjectedScrollEvent {
                target: key(2),
                x: 0.0,
                y: 80.0,
            })
        );
        assert_eq!(projection.apply_wheel_delta(key(3), 0.0, 0.0), None);
    }

    #[test]
    fn typed_scrollbar_style_is_retained_for_gpui_base_projection() {
        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let view = atoms.intern("view");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement {
                            id: key(2),
                            tag: view,
                        },
                        Op::SetScrollbarStyle {
                            id: key(2),
                            visibility: 1,
                            hide_delay: 700.0,
                            fade_duration: 160.0,
                            thickness: 12.0,
                            margin: 2.0,
                            min_thumb_length: 36.0,
                            radius: 6.0,
                            colors: [1, 2, 3, 4],
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert_eq!(
            projection.tree().node(key(2)).unwrap().scrollbar_style,
            Some(crate::tree::ProjectedScrollbarStyle {
                visibility: 1,
                hide_delay: 700.0,
                fade_duration: 160.0,
                thickness: 12.0,
                margin: 2.0,
                min_thumb_length: 36.0,
                radius: 6.0,
                colors: [1, 2, 3, 4],
            })
        );
        assert!(projection.protocol_gaps().is_empty());
    }

    #[test]
    fn vector_path_data_decodes_once_into_the_gpui_projection() {
        let mut bytes = vec![0_u8; 60];
        bytes[0..4].copy_from_slice(&0x3150_4257_u32.to_le_bytes());
        bytes[4..6].copy_from_slice(&1_u16.to_le_bytes());
        bytes[8..12].copy_from_slice(&2_u32.to_le_bytes());
        let byte_len = bytes.len() as u32;
        bytes[12..16].copy_from_slice(&byte_len.to_le_bytes());
        bytes[16..20].copy_from_slice(&0xff00_00ff_u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&1_f32.to_le_bytes());
        bytes[32..36].copy_from_slice(&4_f32.to_le_bytes());
        bytes[36] = 1;
        bytes[40..44].copy_from_slice(&1_f32.to_le_bytes());
        bytes[44..48].copy_from_slice(&2_f32.to_le_bytes());
        bytes[48] = 2;
        bytes[52..56].copy_from_slice(&9_f32.to_le_bytes());
        bytes[56..60].copy_from_slice(&10_f32.to_le_bytes());

        let mut projection = GpuiProjection::new();
        let mut atoms = AtomPool::default();
        let tag = atoms.intern("vector-path");
        projection
            .apply_ops(
                &Frame {
                    seq: 1,
                    ops: vec![
                        Op::CreateElement { id: key(2), tag },
                        Op::SetGraphicData {
                            id: key(2),
                            kind: wabou_protocol::GRAPHIC_DATA_VECTOR_PATH,
                            data: &bytes,
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();

        assert!(
            projection
                .tree()
                .node(key(2))
                .unwrap()
                .vector_path
                .is_some()
        );
        assert!(projection.protocol_gaps().is_empty());
    }

    #[test]
    fn style_dirty_kind_keeps_paint_changes_out_of_layout() {
        let previous = crate::gpui::Style::default();
        let mut next = previous.clone();
        next.opacity = Some(0.5);

        assert_eq!(
            projected_style_dirty_kind(&previous, &next),
            DirtyKind::PAINT
        );
    }

    #[test]
    fn style_dirty_kind_marks_layout_and_text_metrics_precisely() {
        let previous = crate::gpui::Style::default();
        let mut layout = previous.clone();
        layout.flex_grow = 1.0;
        assert_eq!(
            projected_style_dirty_kind(&previous, &layout),
            DirtyKind::LAYOUT | DirtyKind::PAINT
        );

        let mut text = previous.clone();
        text.text.font_weight = Some(crate::gpui::FontWeight::BOLD);
        assert_eq!(
            projected_style_dirty_kind(&previous, &text),
            DirtyKind::LAYOUT | DirtyKind::TEXT | DirtyKind::PAINT
        );
    }
}
