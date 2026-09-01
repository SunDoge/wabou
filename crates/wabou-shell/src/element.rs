use std::{cell::RefCell, collections::BTreeMap, ops::Range, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Element, ElementId, FocusHandle, GlobalElementId,
    HighlightStyle, Hitbox, HitboxBehavior, InspectorElementId, IntoElement, KeyDownEvent,
    KeyUpEvent, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Overflow,
    Pixels, ScrollDelta, ScrollWheelEvent, StyledText, TouchPhase, UniformListScrollHandle,
    Visibility, Window, div, prelude::*, uniform_list,
};

use crate::ProjectionSnapshot;
use crate::{
    GpuiNodeKeyExt, NodeKey, ProjectedInputEvent, ProjectedInputSink, ProjectedKeyEvent,
    ProjectedKeyPhase, ProjectedNode, ProjectedNodeKind, ProjectedPointerButton,
    ProjectedPointerEvent, ProjectedPointerPhase, ProjectedScrollEvent, ProjectedTextInputState,
    ProjectedWheelEvent, ProjectedWheelPhase, ProjectionError, TextSelectionPolicy,
};
use wabou_protocol::{TEXT_BEHAVIOR_AGGREGATE_DIRECT, TEXT_BEHAVIOR_AGGREGATE_STYLED};

/// Produces a GPUI-owned native control for a retained Wabou node.
///
/// The callback is evaluated once while materializing a frame. It lets the
/// runtime preserve platform control state in `Entity<T>` without moving that
/// state into the lightweight projection cache.
pub type ProjectedNativeElementFactory = Rc<dyn Fn(NodeKey) -> Option<AnyElement>>;

pub(crate) type ProjectedLayoutBounds = Rc<RefCell<BTreeMap<NodeKey, Bounds<Pixels>>>>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ProjectedGraphicPaintState {
    SvgPainted {
        bounds: Bounds<Pixels>,
        color: gpui::Hsla,
    },
    SvgFailed {
        message: String,
    },
}

pub(crate) type ProjectedGraphicPaintStates =
    Rc<RefCell<BTreeMap<NodeKey, ProjectedGraphicPaintState>>>;

/// Stable scroll state retained independently from GPUI's per-frame elements.
#[derive(Clone, Debug, Default)]
pub struct ProjectedScrollHandle(Rc<RefCell<ProjectedScrollState>>);

#[derive(Clone, Copy, Debug, Default)]
struct ProjectedScrollState {
    offset: gpui::Point<Pixels>,
    max_offset: gpui::Point<Pixels>,
    viewport: Bounds<Pixels>,
    has_extent: bool,
}

impl ProjectedScrollHandle {
    /// Current logical, positive scroll position.
    pub fn position(&self) -> gpui::Point<Pixels> {
        let offset = self.0.borrow().offset;
        gpui::point(-offset.x, -offset.y)
    }

    /// Set logical scroll coordinates. Non-finite axes retain their old value.
    pub fn scroll_to(&self, x: f32, y: f32) -> bool {
        self.update_logical(x, y, false)
    }

    /// Add logical scroll coordinates. Non-finite axes retain their old value.
    pub fn scroll_by(&self, x: f32, y: f32) -> bool {
        self.update_logical(x, y, true)
    }

    fn update_logical(&self, x: f32, y: f32, relative: bool) -> bool {
        let mut state = self.0.borrow_mut();
        let old = state.offset;
        let current = gpui::point(-state.offset.x, -state.offset.y);
        let next_x = if x.is_finite() {
            if relative {
                f32::from(current.x) + x
            } else {
                x
            }
        } else {
            f32::from(current.x)
        };
        let next_y = if y.is_finite() {
            if relative {
                f32::from(current.y) + y
            } else {
                y
            }
        } else {
            f32::from(current.y)
        };
        state.offset.x = -if state.has_extent {
            gpui::px(next_x).clamp(Pixels::ZERO, state.max_offset.x)
        } else {
            gpui::px(next_x).max(Pixels::ZERO)
        };
        state.offset.y = -if state.has_extent {
            gpui::px(next_y).clamp(Pixels::ZERO, state.max_offset.y)
        } else {
            gpui::px(next_y).max(Pixels::ZERO)
        };
        state.offset != old
    }

    fn set_max_offset(&self, max_offset: gpui::Point<Pixels>) {
        let mut state = self.0.borrow_mut();
        state.max_offset = max_offset;
        state.has_extent = true;
        state.offset.x = state.offset.x.clamp(-max_offset.x, Pixels::ZERO);
        state.offset.y = state.offset.y.clamp(-max_offset.y, Pixels::ZERO);
    }

    fn set_viewport(&self, viewport: Bounds<Pixels>) {
        self.0.borrow_mut().viewport = viewport;
    }

    fn offset(&self) -> gpui::Point<Pixels> {
        self.0.borrow().offset
    }

    fn apply_native_delta(
        &self,
        delta: gpui::Point<Pixels>,
        scroll_x: bool,
        scroll_y: bool,
    ) -> bool {
        let mut state = self.0.borrow_mut();
        let old = state.offset;
        if scroll_x {
            state.offset.x = (state.offset.x + delta.x).clamp(-state.max_offset.x, Pixels::ZERO);
        }
        if scroll_y {
            state.offset.y = (state.offset.y + delta.y).clamp(-state.max_offset.y, Pixels::ZERO);
        }
        state.offset != old
    }
}

impl gpui_base::ScrollbarHandle for ProjectedScrollHandle {
    fn viewport_bounds(&self) -> Bounds<Pixels> {
        self.0.borrow().viewport
    }

    fn offset(&self) -> gpui::Point<Pixels> {
        self.offset()
    }

    fn set_offset(&self, offset: gpui::Point<Pixels>) {
        let mut state = self.0.borrow_mut();
        state.offset.x = offset.x.clamp(-state.max_offset.x, Pixels::ZERO);
        state.offset.y = offset.y.clamp(-state.max_offset.y, Pixels::ZERO);
    }

    fn content_size(&self) -> gpui::Size<Pixels> {
        let state = self.0.borrow();
        gpui::size(
            state.viewport.size.width + state.max_offset.x,
            state.viewport.size.height + state.max_offset.y,
        )
    }
}

#[derive(Clone, Default)]
pub(crate) struct ProjectedElementContext {
    pub(crate) input: Option<ProjectedInputSink>,
    pub(crate) root_focus: Option<FocusHandle>,
    pub(crate) text_input: Option<ProjectedTextInputState>,
    pub(crate) native: Option<ProjectedNativeElementFactory>,
    pub(crate) layout_bounds: Option<ProjectedLayoutBounds>,
    pub(crate) graphic_paint_states: Option<ProjectedGraphicPaintStates>,
    pub(crate) scroll_handles: Option<Rc<BTreeMap<NodeKey, ProjectedScrollHandle>>>,
    pub(crate) uniform_list_handles: Option<Rc<BTreeMap<NodeKey, UniformListScrollHandle>>>,
    pub(crate) text_selections: Option<crate::text_selection::ProjectedTextSelections>,
    pub(crate) text_selection_policy: Option<TextSelectionPolicy>,
}

impl ProjectedElementContext {
    fn for_child(&self) -> Self {
        Self {
            root_focus: None,
            text_input: None,
            ..self.clone()
        }
    }
}

/// A lightweight GPUI element generated from one Wabou retained node.
///
/// GPUI drops element objects after each frame. Stable state survives through
/// [`Element::id`], which is derived losslessly from Wabou's generational key.
pub struct ProjectedElement {
    key: NodeKey,
    style: gpui::Style,
    children: Vec<AnyElement>,
    input: Option<ProjectedInputSink>,
    /// Whether this exact node should own a GPUI pointer hitbox.
    ///
    /// Descendants without pointer behavior must not steal hover/press state
    /// from an interactive ancestor merely because every projected node shares
    /// the same event bridge.
    hit_testable: bool,
    suppress_text_selection: bool,
    root_focus: Option<FocusHandle>,
    text_input: Option<ProjectedTextInputState>,
    layout_bounds: Option<ProjectedLayoutBounds>,
    graphic_paint_states: Option<ProjectedGraphicPaintStates>,
    scroll: Option<ProjectedScrollHandle>,
    scroll_x: bool,
    scroll_y: bool,
    transform: [f32; 6],
    svg_source: Option<crate::tree::ProjectedSvgSource>,
    vector_path: Option<std::sync::Arc<crate::vector_path::ProjectedVectorPath>>,
    accessibility: Option<ProjectedAccessibility>,
    scrollbar_style: Option<crate::tree::ProjectedScrollbarStyle>,
}

#[derive(Clone)]
struct ProjectedAccessibility {
    role: gpui::accesskit::Role,
    label: Option<gpui::SharedString>,
    value: Option<gpui::SharedString>,
    disabled: bool,
    selected: Option<bool>,
    expanded: Option<bool>,
    toggled: Option<gpui::accesskit::Toggled>,
    numeric_value: Option<f64>,
    min_numeric_value: Option<f64>,
    max_numeric_value: Option<f64>,
    orientation: Option<gpui::accesskit::Orientation>,
}

fn has_pointer_listener(node: &crate::tree::ProjectedNode) -> bool {
    use wabou_protocol::event;

    node.listeners.iter().any(|event_type| {
        matches!(
            *event_type,
            event::CLICK
                | event::DBLCLICK
                | event::CONTEXTMENU
                | event::POINTERDOWN
                | event::POINTERUP
                | event::POINTERMOVE
                | event::POINTERENTER
                | event::POINTERLEAVE
                | event::POINTEROVER
                | event::POINTEROUT
                | event::POINTERCANCEL
                | event::WHEEL
        )
    })
}

fn parse_bool(value: Option<&gpui::SharedString>) -> Option<bool> {
    match value.map(AsRef::as_ref) {
        Some("true") => Some(true),
        Some("false") => Some(false),
        _ => None,
    }
}

fn parse_toggled(value: Option<&gpui::SharedString>) -> Option<gpui::accesskit::Toggled> {
    match value.map(AsRef::as_ref) {
        Some("true") => Some(gpui::accesskit::Toggled::True),
        Some("false") => Some(gpui::accesskit::Toggled::False),
        Some("mixed") => Some(gpui::accesskit::Toggled::Mixed),
        _ => None,
    }
}

fn accesskit_role(role: wabou_accessibility::SemanticRole) -> gpui::accesskit::Role {
    use gpui::accesskit::Role;
    use wabou_accessibility::SemanticRole as Semantic;
    match role {
        Semantic::Generic | Semantic::Group => Role::Group,
        Semantic::Label => Role::Label,
        Semantic::Heading => Role::Heading,
        Semantic::Button => Role::Button,
        Semantic::TextInput => Role::TextInput,
        Semantic::Image => Role::Image,
        Semantic::RadioGroup => Role::RadioGroup,
        Semantic::Region => Role::Region,
        Semantic::Link => Role::Link,
        Semantic::List => Role::List,
        Semantic::ListItem => Role::ListItem,
        Semantic::Dialog => Role::Dialog,
        Semantic::AlertDialog => Role::AlertDialog,
        Semantic::Alert => Role::Alert,
        Semantic::Status => Role::Status,
        Semantic::Tooltip => Role::Tooltip,
        Semantic::CheckBox => Role::CheckBox,
        Semantic::RadioButton => Role::RadioButton,
        Semantic::Switch => Role::Switch,
        Semantic::ComboBox => Role::ComboBox,
        Semantic::ListBox => Role::ListBox,
        Semantic::Option => Role::ListBoxOption,
        Semantic::Menu => Role::Menu,
        Semantic::MenuBar => Role::MenuBar,
        Semantic::MenuItem => Role::MenuItem,
        Semantic::Tree => Role::Tree,
        Semantic::TreeItem => Role::TreeItem,
        Semantic::Toolbar => Role::Toolbar,
        Semantic::Table => Role::Table,
        Semantic::Row => Role::Row,
        Semantic::Cell => Role::Cell,
        Semantic::ColumnHeader => Role::ColumnHeader,
        Semantic::RowHeader => Role::RowHeader,
        Semantic::Separator => Role::Splitter,
        Semantic::Slider => Role::Slider,
        Semantic::SpinButton => Role::SpinButton,
        Semantic::ProgressBar => Role::ProgressIndicator,
        Semantic::TabList => Role::TabList,
        Semantic::Tab => Role::Tab,
        Semantic::TabPanel => Role::TabPanel,
        Semantic::Grid => Role::Grid,
        Semantic::GridCell => Role::GridCell,
    }
}

fn projected_accessibility(node: &ProjectedNode) -> Option<ProjectedAccessibility> {
    if parse_bool(node.attributes.get("aria-hidden")) == Some(true) {
        return None;
    }
    let role = node
        .attributes
        .get("role")
        .and_then(|role| wabou_accessibility::SemanticRole::from_name(role))?;
    let number = |name: &str| {
        node.attributes
            .get(name)
            .and_then(|value| value.parse::<f64>().ok())
    };
    Some(ProjectedAccessibility {
        role: accesskit_role(role),
        label: node.attributes.get("aria-label").cloned(),
        value: node
            .attributes
            .get("aria-valuetext")
            .or_else(|| node.attributes.get("value"))
            .cloned(),
        disabled: node.attributes.contains_key("disabled")
            || parse_bool(node.attributes.get("aria-disabled")) == Some(true),
        selected: parse_bool(node.attributes.get("aria-selected")),
        expanded: parse_bool(node.attributes.get("aria-expanded")),
        toggled: parse_toggled(
            node.attributes
                .get("aria-checked")
                .or_else(|| node.attributes.get("aria-pressed")),
        ),
        numeric_value: number("aria-valuenow"),
        min_numeric_value: number("aria-valuemin"),
        max_numeric_value: number("aria-valuemax"),
        orientation: match node.attributes.get("aria-orientation").map(AsRef::as_ref) {
            Some("horizontal") => Some(gpui::accesskit::Orientation::Horizontal),
            Some("vertical") => Some(gpui::accesskit::Orientation::Vertical),
            _ => None,
        },
    })
}

pub struct ProjectedPrepaintState {
    hitbox: Option<Hitbox>,
    paint_bounds: Bounds<Pixels>,
    scrollbar: Option<AnyElement>,
}

pub struct ProjectedRequestLayoutState {
    child_layouts: Vec<LayoutId>,
    scrollbar_layout: Option<LayoutId>,
    scrollbar: Option<AnyElement>,
}

impl ProjectedElement {
    pub(crate) fn from_tree(
        tree: ProjectionSnapshot,
        key: NodeKey,
        context: ProjectedElementContext,
        ancestor_blocked: bool,
    ) -> Result<Self, ProjectionError> {
        let node = tree.node(key).ok_or(ProjectionError::MissingNode(key))?;
        let interaction_blocked = ancestor_blocked || node.interaction_blocked;
        let text_selection_policy = node.text_selection.unwrap_or(
            context
                .text_selection_policy
                .unwrap_or(TextSelectionPolicy::Text),
        );
        let mut child_context = context.for_child();
        child_context.text_selection_policy = Some(text_selection_policy);
        let native_child = context.native.as_ref().and_then(|factory| factory(key));
        let has_native_child = native_child.is_some();
        let mut children =
            Vec::with_capacity(node.children.len() + usize::from(node.text.is_some()));
        let is_uniform_list = matches!(
            &node.kind,
            ProjectedNodeKind::Element(tag) if tag.as_ref() == "virtual-list"
        );
        if is_uniform_list {
            let row_keys = node.children.clone();
            let row_count = row_keys.len();
            let row_tree = tree.clone();
            let row_context = child_context.clone();
            let list = uniform_list(
                format!("wabou-uniform-list-{}-{}", key.hi, key.lo),
                row_count,
                move |range, _window, _cx| {
                    range
                        .map(|index| {
                            Self::from_tree(
                                row_tree.clone(),
                                row_keys[index],
                                row_context.clone(),
                                interaction_blocked,
                            )
                            .expect("uniform-list rows belong to the validated snapshot")
                        })
                        .collect::<Vec<_>>()
                },
            );
            let list = if let Some(handle) = context
                .uniform_list_handles
                .as_ref()
                .and_then(|handles| handles.get(&key))
            {
                list.track_scroll(handle).size_full()
            } else {
                list.size_full()
            };
            children.push(list.into_any_element());
        } else if let Some(native_child) = native_child {
            children.push(native_child);
        } else if let Some((text, styled_text)) = projected_text_element(&tree, node) {
            let selectable = (text_selection_policy != TextSelectionPolicy::None)
                .then(|| {
                    context
                        .text_selections
                        .as_ref()
                        .and_then(|selections| selections.get(&key))
                })
                .flatten();
            children.push(if let Some(selection) = selectable {
                crate::text_selection::selectable_text_element(selection.clone(), text, styled_text)
            } else {
                div().child(styled_text).into_any_element()
            });
        }
        if let Some(image) = &node.image {
            children.push(gpui::img(image.clone()).size_full().into_any_element());
        }
        let ordinary_children: &[NodeKey] = if is_uniform_list { &[] } else { &node.children };
        for child in ordinary_children {
            if node.text_behavior & TEXT_BEHAVIOR_AGGREGATE_DIRECT != 0
                && tree.node(*child).is_some_and(|child| {
                    child.kind == ProjectedNodeKind::Text
                        || node.text_behavior & TEXT_BEHAVIOR_AGGREGATE_STYLED != 0
                            && matches!(
                                &child.kind,
                                ProjectedNodeKind::Element(tag)
                                    if tag.as_ref() == "text-span"
                            )
                })
            {
                continue;
            }
            let projected = Self::from_tree(
                tree.clone(),
                *child,
                child_context.clone(),
                interaction_blocked,
            )?;
            let priority = tree
                .node(*child)
                .ok_or(ProjectionError::MissingNode(*child))?
                .draw_priority();
            if priority == 0 {
                children.push(projected.into_any_element());
            } else {
                children.push(
                    gpui::deferred(projected)
                        .with_priority(priority)
                        .into_any_element(),
                );
            }
        }
        let hit_testable = !has_native_child
            && !interaction_blocked
            && node.pointer_events
            && (context.root_focus.is_some()
                || node.focus_order.is_some()
                || has_pointer_listener(node));
        Ok(Self {
            key,
            style: node.style.clone(),
            children,
            input: (!interaction_blocked && node.pointer_events)
                .then_some(context.input)
                .flatten(),
            hit_testable,
            suppress_text_selection: text_selection_policy == TextSelectionPolicy::None,
            root_focus: context.root_focus,
            text_input: context.text_input,
            layout_bounds: context.layout_bounds,
            graphic_paint_states: context.graphic_paint_states,
            scroll: context
                .scroll_handles
                .as_ref()
                .and_then(|handles| handles.get(&key).cloned()),
            scroll_x: node.style.overflow.x == Overflow::Scroll,
            scroll_y: node.style.overflow.y == Overflow::Scroll,
            transform: node.transform,
            svg_source: node.svg_source.clone(),
            vector_path: node.vector_path.clone(),
            accessibility: projected_accessibility(node),
            scrollbar_style: node.scrollbar_style,
        })
    }

    fn translation(&self) -> gpui::Point<Pixels> {
        if self.svg_source.is_some() {
            return gpui::Point::default();
        }
        if self.transform[0] == 1.0
            && self.transform[1] == 0.0
            && self.transform[2] == 0.0
            && self.transform[3] == 1.0
        {
            gpui::point(gpui::px(self.transform[4]), gpui::px(self.transform[5]))
        } else {
            gpui::point(gpui::px(0.0), gpui::px(0.0))
        }
    }

    fn svg_transformation(
        &self,
        bounds: Bounds<Pixels>,
        scale_factor: f32,
    ) -> gpui::TransformationMatrix {
        let [a, b, c, d, e, f] = self.transform;
        let center = bounds.center().scale(scale_factor);
        gpui::TransformationMatrix {
            rotation_scale: [[a, c], [b, d]],
            translation: [
                center.x.0 + e * scale_factor - a * center.x.0 - c * center.y.0,
                center.y.0 + f * scale_factor - b * center.x.0 - d * center.y.0,
            ],
        }
    }

    fn scrollbar_element(&self) -> Option<AnyElement> {
        // Pure layout materialization has no current GPUI view. Base's
        // scrollbar owns keyed interaction state and is therefore projected
        // only for the interactive application tree.
        self.input.as_ref()?;
        let scroll = self.scroll.as_ref()?;
        let style = self.scrollbar_style.unwrap_or_default();
        if style.visibility == 2 {
            return None;
        }
        let axis = match (self.scroll_x, self.scroll_y) {
            (true, true) => gpui_base::ScrollbarAxis::Both,
            (true, false) => gpui_base::ScrollbarAxis::Horizontal,
            (false, true) => gpui_base::ScrollbarAxis::Vertical,
            (false, false) => return None,
        };
        let mode = if style.visibility == 1 {
            gpui_base::ScrollbarMode::Always
        } else {
            gpui_base::ScrollbarMode::Scrolling
        };
        let radius = if style.radius < 0.0 {
            style.thickness / 2.0
        } else {
            style.radius
        };
        let thumb_width = (style.thickness - style.margin * 2.0).max(1.0);
        let track = gpui::rgb_to_hsla(gpui::rgba(style.colors[0]));
        let thumb = gpui::rgb_to_hsla(gpui::rgba(style.colors[1]));
        let hover = gpui::rgb_to_hsla(gpui::rgba(style.colors[2]));
        let active = gpui::rgb_to_hsla(gpui::rgba(style.colors[3]));
        let bar = gpui_base::Scrollbar::new(scroll)
            .id(format!("wabou-scrollbar-{}-{}", self.key.lo, self.key.hi))
            .axis(axis)
            .mode(mode)
            .styles(|styles| {
                styles
                    .track(|value| value.bg(track).width(gpui::px(style.thickness)))
                    .track_hover(|value| value.bg(track).width(gpui::px(style.thickness)))
                    .track_active(|value| value.bg(track).width(gpui::px(style.thickness)))
                    .thumb(|value| {
                        value
                            .bg(thumb)
                            .width(gpui::px(thumb_width))
                            .inset(gpui::px(style.margin))
                            .radius(gpui::px(radius))
                            .min_length(gpui::px(style.min_thumb_length))
                    })
                    .thumb_hover(|value| value.bg(hover))
                    .thumb_active(|value| value.bg(active))
            });
        Some(div().absolute().inset_0().child(bar).into_any_element())
    }
}

pub(crate) fn projected_text(
    tree: &ProjectionSnapshot,
    node: &ProjectedNode,
) -> Option<gpui::SharedString> {
    if let Some(text) = &node.text {
        return Some(text.clone());
    }
    if node.text_behavior & TEXT_BEHAVIOR_AGGREGATE_DIRECT != 0
        && node.text_behavior & TEXT_BEHAVIOR_AGGREGATE_STYLED == 0
    {
        let mut text = String::new();
        for child in &node.children {
            let Some(child) = tree.node(*child) else {
                continue;
            };
            if child.kind == ProjectedNodeKind::Text
                && let Some(value) = &child.text
            {
                text.push_str(value);
            }
        }
        if !text.is_empty() {
            return Some(text.into());
        }
    }
    if node.text_behavior & TEXT_BEHAVIOR_AGGREGATE_STYLED != 0 {
        fn append_styled_text(tree: &ProjectionSnapshot, key: NodeKey, output: &mut String) {
            let Some(node) = tree.node(key) else {
                return;
            };
            if node.kind == ProjectedNodeKind::Text {
                if let Some(value) = &node.text {
                    output.push_str(value);
                }
                return;
            }
            let is_span = matches!(
                &node.kind,
                ProjectedNodeKind::Element(tag) if tag.as_ref() == "text-span"
            );
            if !is_span {
                return;
            }
            for child in &node.children {
                append_styled_text(tree, *child, output);
            }
        }

        let mut text = String::new();
        for child in &node.children {
            append_styled_text(tree, *child, &mut text);
        }
        if !text.is_empty() {
            return Some(text.into());
        }
    }
    {
        let ProjectedNodeKind::Element(tag) = &node.kind else {
            return None;
        };
        matches!(tag.as_ref(), "input" | "textarea")
            .then(|| {
                node.attributes
                    .get("value")
                    .filter(|value| !value.is_empty())
                    .or_else(|| node.attributes.get("placeholder"))
            })
            .flatten()
            .cloned()
    }
}

struct ProjectedStyledText {
    text: String,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
    font_families: Vec<(Range<usize>, gpui::SharedString)>,
}

fn projected_styled_text(
    tree: &ProjectionSnapshot,
    node: &ProjectedNode,
) -> Option<ProjectedStyledText> {
    if node.text_behavior & TEXT_BEHAVIOR_AGGREGATE_STYLED == 0 {
        return None;
    }

    #[derive(Clone, Default)]
    struct SpanStyle {
        highlight: HighlightStyle,
        font_family: Option<gpui::SharedString>,
    }

    fn append(
        tree: &ProjectionSnapshot,
        key: NodeKey,
        inherited: &SpanStyle,
        output: &mut ProjectedStyledText,
    ) {
        let Some(node) = tree.node(key) else {
            return;
        };
        if node.kind == ProjectedNodeKind::Text {
            let Some(value) = &node.text else {
                return;
            };
            let start = output.text.len();
            output.text.push_str(value);
            let range = start..output.text.len();
            if inherited.highlight != HighlightStyle::default() {
                output.highlights.push((range.clone(), inherited.highlight));
            }
            if let Some(family) = &inherited.font_family {
                output.font_families.push((range, family.clone()));
            }
            return;
        }
        let is_span = matches!(
            &node.kind,
            ProjectedNodeKind::Element(tag) if tag.as_ref() == "text-span"
        );
        if !is_span {
            return;
        }

        let mut style = inherited.clone();
        if let Some(color) = node.style.text.color {
            style.highlight.color = Some(color);
        }
        if let Some(weight) = node.style.text.font_weight {
            style.highlight.font_weight = Some(weight);
        }
        if let Some(font_style) = node.style.text.font_style {
            style.highlight.font_style = Some(font_style);
        }
        if let Some(background) = node
            .style
            .background
            .as_ref()
            .and_then(gpui::Fill::color)
            .and_then(|background| background.as_solid())
        {
            style.highlight.background_color = Some(background);
        }
        if let Some(underline) = node.style.text.underline {
            style.highlight.underline = Some(underline);
        }
        if let Some(strikethrough) = node.style.text.strikethrough {
            style.highlight.strikethrough = Some(strikethrough);
        }
        if let Some(opacity) = node.style.opacity
            && opacity < 1.0
        {
            style.highlight.fade_out = Some(1.0 - opacity);
        }
        if let Some(family) = &node.style.text.font_family {
            style.font_family = Some(family.clone());
        }
        for child in &node.children {
            append(tree, *child, &style, output);
        }
    }

    let mut output = ProjectedStyledText {
        text: String::new(),
        highlights: Vec::new(),
        font_families: Vec::new(),
    };
    for child in &node.children {
        append(tree, *child, &SpanStyle::default(), &mut output);
    }
    (!output.text.is_empty()).then_some(output)
}

fn projected_text_element(
    tree: &ProjectionSnapshot,
    node: &ProjectedNode,
) -> Option<(gpui::SharedString, StyledText)> {
    if let Some(styled) = projected_styled_text(tree, node) {
        let text: gpui::SharedString = styled.text.into();
        let element = StyledText::new(text.clone())
            .with_highlights(styled.highlights)
            .with_font_family_overrides(styled.font_families);
        return Some((text, element));
    }
    projected_text(tree, node).map(|text| {
        let element = StyledText::new(text.clone());
        (text, element)
    })
}

fn key_event(
    phase: ProjectedKeyPhase,
    keystroke: &gpui::Keystroke,
    repeat: bool,
) -> ProjectedKeyEvent {
    ProjectedKeyEvent {
        phase,
        key: keystroke.key.clone(),
        key_char: keystroke.key_char.clone(),
        repeat,
        shift: keystroke.modifiers.shift,
        control: keystroke.modifiers.control,
        alt: keystroke.modifiers.alt,
        platform: keystroke.modifiers.platform,
    }
}

fn pointer_button(button: MouseButton) -> ProjectedPointerButton {
    match button {
        MouseButton::Left => ProjectedPointerButton::Primary,
        MouseButton::Middle => ProjectedPointerButton::Auxiliary,
        MouseButton::Right => ProjectedPointerButton::Secondary,
        MouseButton::Navigate(_) => ProjectedPointerButton::Other,
    }
}

fn pointer_event(
    key: NodeKey,
    phase: ProjectedPointerPhase,
    position: gpui::Point<Pixels>,
    bounds: Bounds<Pixels>,
    button: Option<MouseButton>,
    modifiers: gpui::Modifiers,
) -> ProjectedPointerEvent {
    ProjectedPointerEvent {
        target: key,
        phase,
        x: position.x.into(),
        y: position.y.into(),
        local_x: (position.x - bounds.origin.x).into(),
        local_y: (position.y - bounds.origin.y).into(),
        button: button.map(pointer_button),
        shift: modifiers.shift,
        control: modifiers.control,
        alt: modifiers.alt,
        platform: modifiers.platform,
    }
}

fn wheel_event(
    key: NodeKey,
    event: &ScrollWheelEvent,
    bounds: Bounds<Pixels>,
) -> ProjectedWheelEvent {
    let (delta_x, delta_y, precise) = match event.delta {
        ScrollDelta::Pixels(delta) => (f32::from(delta.x), f32::from(delta.y), true),
        ScrollDelta::Lines(delta) => (delta.x, delta.y, false),
    };
    ProjectedWheelEvent {
        target: key,
        x: event.position.x.into(),
        y: event.position.y.into(),
        local_x: (event.position.x - bounds.origin.x).into(),
        local_y: (event.position.y - bounds.origin.y).into(),
        delta_x,
        delta_y,
        precise,
        phase: match event.touch_phase {
            TouchPhase::Started => ProjectedWheelPhase::Started,
            TouchPhase::Moved => ProjectedWheelPhase::Changed,
            TouchPhase::Ended => ProjectedWheelPhase::Ended,
            TouchPhase::Cancelled => ProjectedWheelPhase::Cancelled,
        },
        shift: event.modifiers.shift,
        control: event.modifiers.control,
        alt: event.modifiers.alt,
        platform: event.modifiers.platform,
    }
}

impl IntoElement for ProjectedElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ProjectedElement {
    type RequestLayoutState = ProjectedRequestLayoutState;
    type PrepaintState = ProjectedPrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(self.key.gpui_element_id())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn a11y_role(&self) -> Option<gpui::accesskit::Role> {
        self.accessibility.as_ref().map(|value| value.role)
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        let Some(value) = &self.accessibility else {
            return;
        };
        if let Some(label) = &value.label {
            node.set_label(label.to_string());
        }
        if let Some(text) = &value.value {
            node.set_value(text.to_string());
        }
        if value.disabled {
            node.set_disabled();
        }
        if let Some(selected) = value.selected {
            node.set_selected(selected);
        }
        if let Some(expanded) = value.expanded {
            node.set_expanded(expanded);
        }
        if let Some(toggled) = value.toggled {
            node.set_toggled(toggled);
        }
        if let Some(number) = value.numeric_value {
            node.set_numeric_value(number);
        }
        if let Some(number) = value.min_numeric_value {
            node.set_min_numeric_value(number);
        }
        if let Some(number) = value.max_numeric_value {
            node.set_max_numeric_value(number);
        }
        if let Some(orientation) = value.orientation {
            node.set_orientation(orientation);
        }
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // Taffy deliberately does not visit descendants of `display: none`.
        // Requesting their GPUI layout anyway creates text layout handles whose
        // measure callbacks never run; prepainting those children later then
        // violates GPUI's measure -> prepaint contract. Treat the whole hidden
        // subtree as absent from both phases.
        if self.style.display == gpui::Display::None {
            let layout_id = window.request_layout(self.style.clone(), [], cx);
            return (
                layout_id,
                ProjectedRequestLayoutState {
                    child_layouts: Vec::new(),
                    scrollbar_layout: None,
                    scrollbar: None,
                },
            );
        }
        // GPUI shapes text during request_layout and stores the resolved color,
        // family, weight, and line metrics in its text runs. Applying inherited
        // refinements only during prepaint/paint leaves those runs at GPUI's
        // default (notably black), even though the projected Style is correct.
        let text_style = self.style.text_style().cloned();
        let (child_layouts, scrollbar, scrollbar_layout) =
            window.with_text_style(text_style, |window| {
                let child_layouts = self
                    .children
                    .iter_mut()
                    .map(|child| child.request_layout(window, cx))
                    .collect::<Vec<_>>();
                let mut scrollbar = self.scrollbar_element();
                let scrollbar_layout = scrollbar
                    .as_mut()
                    .map(|scrollbar| scrollbar.request_layout(window, cx));
                (child_layouts, scrollbar, scrollbar_layout)
            });
        let mut layout_children = child_layouts.clone();
        layout_children.extend(scrollbar_layout);
        let layout_id = window.request_layout(self.style.clone(), layout_children, cx);
        (
            layout_id,
            ProjectedRequestLayoutState {
                child_layouts,
                scrollbar_layout,
                scrollbar,
            },
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if let Some(layout_bounds) = &self.layout_bounds {
            layout_bounds.borrow_mut().insert(self.key, bounds);
        }
        if self.style.display == gpui::Display::None {
            return ProjectedPrepaintState {
                hitbox: None,
                paint_bounds: bounds,
                scrollbar: None,
            };
        }
        let translation = self.translation();
        let paint_bounds = Bounds {
            origin: bounds.origin + translation,
            size: bounds.size,
        };
        if let Some(scroll) = &self.scroll {
            scroll.set_viewport(paint_bounds);
        }
        let text_style = self.style.text_style().cloned();
        let overflow_mask = self.style.overflow_mask(paint_bounds, window.rem_size());
        if let Some(focus) = &self.root_focus {
            window.set_focus_handle(focus, cx);
        }
        let scroll_offset = if let Some(scroll) = &self.scroll {
            let mut child_min = gpui::point(Pixels::MAX, Pixels::MAX);
            let mut child_max = gpui::Point::default();
            for child_layout in &request_layout.child_layouts {
                let child_bounds = window.layout_bounds(*child_layout);
                child_min = child_min.min(&child_bounds.origin);
                child_max = child_max.max(&child_bounds.bottom_right());
            }
            let content_size = if request_layout.child_layouts.is_empty() {
                bounds.size
            } else {
                gpui::Size::from(child_max - child_min)
            };
            let padding = self
                .style
                .padding
                .to_pixels(bounds.size.into(), window.rem_size());
            let content_size = gpui::size(
                content_size.width + padding.left + padding.right,
                content_size.height + padding.top + padding.bottom,
            );
            scroll.set_max_offset(gpui::point(
                if self.scroll_x {
                    (content_size.width - bounds.size.width).max(Pixels::ZERO)
                } else {
                    Pixels::ZERO
                },
                if self.scroll_y {
                    (content_size.height - bounds.size.height).max(Pixels::ZERO)
                } else {
                    Pixels::ZERO
                },
            ));
            scroll.offset()
        } else {
            gpui::Point::default()
        };
        let hitbox = (self.hit_testable || self.scroll_x || self.scroll_y)
            .then(|| window.insert_hitbox(paint_bounds, HitboxBehavior::Normal));
        window.with_text_style(text_style, |window| {
            window.with_content_mask(overflow_mask, |window| {
                window.with_element_offset(translation + scroll_offset, |window| {
                    for child in &mut self.children {
                        child.prepaint(window, cx);
                    }
                });
            });
        });
        let mut scrollbar = request_layout.scrollbar.take();
        if let (Some(scrollbar), Some(_layout)) = (&mut scrollbar, request_layout.scrollbar_layout)
        {
            window.with_element_offset(translation, |window| scrollbar.prepaint(window, cx));
        }
        ProjectedPrepaintState {
            hitbox,
            paint_bounds,
            scrollbar,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.style.display == gpui::Display::None || self.style.visibility == Visibility::Hidden
        {
            return;
        }

        let bounds = prepaint.paint_bounds;
        if self.hit_testable
            && let (Some(input), Some(hitbox)) = (&self.input, prepaint.hitbox.as_ref())
        {
            let down_input = input.clone();
            let down_hitbox = hitbox.clone();
            let key = self.key;
            let suppress_text_selection = self.suppress_text_selection;
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && down_hitbox.is_hovered(window) {
                    if suppress_text_selection && event.button == MouseButton::Left {
                        // Stopping propagation alone is not a sufficient text-selection
                        // contract: the window selection layer prepares the gesture during
                        // capture, before this projected control owns it. Mark the gesture
                        // explicitly so removing a popup during the matching click cannot
                        // turn the same press into a selection on the newly exposed content.
                        gpui_base::GlobalState::suppress_text_selection(cx);
                    }
                    down_input(
                        ProjectedInputEvent::Pointer(pointer_event(
                            key,
                            ProjectedPointerPhase::Down,
                            event.position,
                            bounds,
                            Some(event.button),
                            event.modifiers,
                        )),
                        cx,
                    );
                    cx.stop_propagation();
                }
            });

            let up_input = input.clone();
            let up_hitbox = hitbox.clone();
            window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && up_hitbox.is_hovered(window) {
                    up_input(
                        ProjectedInputEvent::Pointer(pointer_event(
                            key,
                            ProjectedPointerPhase::Up,
                            event.position,
                            bounds,
                            Some(event.button),
                            event.modifiers,
                        )),
                        cx,
                    );
                    cx.stop_propagation();
                }
            });

            let move_input = input.clone();
            let move_hitbox = hitbox.clone();
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && move_hitbox.is_hovered(window) {
                    move_input(
                        ProjectedInputEvent::Pointer(pointer_event(
                            key,
                            ProjectedPointerPhase::Move,
                            event.position,
                            bounds,
                            event.pressed_button,
                            event.modifiers,
                        )),
                        cx,
                    );
                    cx.stop_propagation();
                }
            });
        }

        if let Some(hitbox) = prepaint.hitbox.as_ref() {
            let wheel_input = self.input.clone();
            let wheel_hitbox = hitbox.clone();
            let key = self.key;
            let scroll = self.scroll.clone();
            let scroll_x = self.scroll_x;
            let scroll_y = self.scroll_y;
            let line_height = window.line_height();
            window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !wheel_hitbox.should_handle_scroll(window) {
                    return;
                }
                if let Some(input) = &wheel_input {
                    input(
                        ProjectedInputEvent::Wheel(wheel_event(key, event, bounds)),
                        cx,
                    );
                }
                let changed = scroll.as_ref().is_some_and(|scroll| {
                    scroll.apply_native_delta(
                        event.delta.pixel_delta(line_height),
                        scroll_x,
                        scroll_y,
                    )
                });
                if changed {
                    if let (Some(input), Some(scroll)) = (&wheel_input, &scroll) {
                        let position = scroll.position();
                        input(
                            ProjectedInputEvent::Scroll(ProjectedScrollEvent {
                                target: key,
                                x: position.x.into(),
                                y: position.y.into(),
                            }),
                            cx,
                        );
                    }
                    window.refresh();
                    cx.stop_propagation();
                }
            });
        }

        if let (Some(input), Some(_focus)) = (&self.input, &self.root_focus) {
            let down_input = input.clone();
            window.on_key_event(move |event: &KeyDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble {
                    let copy_shortcut = (event.keystroke.modifiers.platform
                        || event.keystroke.modifiers.control)
                        && event.keystroke.key.eq_ignore_ascii_case("c");
                    if copy_shortcut && gpui_base::TextSelection::has_selection(window, cx) {
                        let selected = gpui_base::TextSelection::selected_text(window, cx);
                        if !selected.is_empty() {
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(selected));
                            cx.stop_propagation();
                            return;
                        }
                    }
                    down_input(
                        ProjectedInputEvent::Key(key_event(
                            ProjectedKeyPhase::Down,
                            &event.keystroke,
                            event.is_held,
                        )),
                        cx,
                    );
                }
            });
            let up_input = input.clone();
            window.on_key_event(move |event: &KeyUpEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble {
                    up_input(
                        ProjectedInputEvent::Key(key_event(
                            ProjectedKeyPhase::Up,
                            &event.keystroke,
                            false,
                        )),
                        cx,
                    );
                }
            });
        }

        if let (Some(input), Some(focus), Some(state)) =
            (&self.input, &self.root_focus, &self.text_input)
            && state.accepts_text
        {
            window.handle_input(
                focus,
                crate::ProjectedInputHandler::new(input.clone(), state.clone()),
                cx,
            );
        }

        let text_style = self.style.text_style().cloned();
        let overflow_mask = self.style.overflow_mask(bounds, window.rem_size());
        let svg_transformation = self
            .svg_source
            .as_ref()
            .map(|_| self.svg_transformation(bounds, window.scale_factor()));
        self.style.paint(bounds, window, cx, |window, cx| {
            window.with_text_style(text_style, |window| {
                window.with_content_mask(overflow_mask, |window| {
                    if let Some(vector_path) = &self.vector_path {
                        vector_path.paint(bounds.origin, window);
                    }
                    if let Some(source) = &self.svg_source {
                        // `currentColor` follows the same inherited text-style
                        // cascade as glyphs. Reading `self.style.text.color`
                        // here would see only declarations authored directly
                        // on the SVG and make inherited icons fall back to
                        // black (effectively invisible on dark surfaces).
                        let svg_color = window.text_style().color;
                        let result = window.paint_svg(
                            bounds,
                            source.cache_key.clone(),
                            Some(&source.bytes),
                            svg_transformation.unwrap_or_default(),
                            svg_color,
                            cx,
                        );
                        if let Some(states) = &self.graphic_paint_states {
                            states.borrow_mut().insert(
                                self.key,
                                match &result {
                                    Ok(()) => ProjectedGraphicPaintState::SvgPainted {
                                        bounds,
                                        color: svg_color,
                                    },
                                    Err(error) => ProjectedGraphicPaintState::SvgFailed {
                                        message: error.to_string(),
                                    },
                                },
                            );
                        }
                        if let Err(error) = result {
                            tracing::warn!(
                                node = ?self.key,
                                cache_key = %source.cache_key,
                                width = f32::from(bounds.size.width),
                                height = f32::from(bounds.size.height),
                                %error,
                                "failed to paint projected SVG"
                            );
                        }
                    }
                    for child in &mut self.children {
                        child.paint(window, cx);
                    }
                });
            });
        });
        if let Some(scrollbar) = &mut prepaint.scrollbar {
            scrollbar.paint(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProjectionTree;
    use std::{cell::RefCell, rc::Rc};

    use gpui::{Context, Keystroke, Modifiers, Style, TestAppContext, point, px};
    use gpui_base::ScrollbarHandle as _;

    #[test]
    fn retained_scroll_handle_adapts_to_gpui_base_without_copying_state() {
        let handle = ProjectedScrollHandle::default();
        handle.set_viewport(Bounds::new(
            point(px(10.0), px(20.0)),
            gpui::size(px(200.0), px(100.0)),
        ));
        handle.set_max_offset(point(px(40.0), px(300.0)));
        gpui_base::ScrollbarHandle::set_offset(&handle, point(px(-12.0), px(-80.0)));

        assert_eq!(handle.position(), point(px(12.0), px(80.0)));
        assert_eq!(handle.content_size(), gpui::size(px(240.0), px(400.0)));
        assert_eq!(
            handle.viewport_bounds(),
            Bounds::new(point(px(10.0), px(20.0)), gpui::size(px(200.0), px(100.0)))
        );
    }

    #[test]
    fn generated_element_uses_the_retained_generational_identity() {
        let key = NodeKey::new(17, 4);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            Style::default(),
            Some("hello".into()),
            crate::ProjectedNodeKind::Text,
        )
        .unwrap();

        let element = ProjectedElement::from_tree(
            tree.snapshot(),
            key,
            ProjectedElementContext::default(),
            false,
        )
        .unwrap();
        assert_eq!(element.id(), Some(key.gpui_element_id()));
    }

    #[test]
    fn inline_svg_affine_matrix_pivots_around_the_border_box_center() {
        let key = NodeKey::new(18, 4);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("svg".into()),
        )
        .unwrap();
        tree.update_svg_source(key, Some(std::sync::Arc::from(b"<svg/>".as_slice())))
            .unwrap();
        tree.update_transform(key, [0.0, 1.0, -1.0, 0.0, 5.0, -3.0])
            .unwrap();

        let element = ProjectedElement::from_tree(
            tree.snapshot(),
            key,
            ProjectedElementContext::default(),
            false,
        )
        .unwrap();
        let bounds = gpui::Bounds::new(point(px(10.0), px(20.0)), gpui::size(px(40.0), px(20.0)));
        let matrix = element.svg_transformation(bounds, 2.0);

        // The center remains the pivot plus the authored logical translation.
        assert_eq!(
            matrix.apply(point(px(60.0), px(60.0))),
            point(px(70.0), px(54.0))
        );
    }

    #[test]
    fn explicit_semantics_are_projected_into_accesskit() {
        let key = NodeKey::new(170, 4);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("view".into()),
        )
        .unwrap();
        for (name, value) in [
            ("role", "slider"),
            ("aria-label", "Volume"),
            ("aria-valuetext", "Quiet"),
            ("aria-valuenow", "25"),
            ("aria-valuemin", "0"),
            ("aria-valuemax", "100"),
            ("aria-disabled", "true"),
            ("aria-selected", "false"),
            ("aria-expanded", "true"),
            ("aria-checked", "mixed"),
            ("aria-orientation", "horizontal"),
        ] {
            tree.update_attribute(key, name.into(), value.into())
                .unwrap();
        }

        let element = ProjectedElement::from_tree(
            tree.snapshot(),
            key,
            ProjectedElementContext::default(),
            false,
        )
        .unwrap();
        assert_eq!(element.a11y_role(), Some(gpui::accesskit::Role::Slider));

        let mut node = gpui::accesskit::Node::new(gpui::accesskit::Role::Slider);
        element.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Volume"));
        assert_eq!(node.value(), Some("Quiet"));
        assert!(node.is_disabled());
        assert_eq!(node.is_selected(), Some(false));
        assert_eq!(node.is_expanded(), Some(true));
        assert_eq!(node.toggled(), Some(gpui::accesskit::Toggled::Mixed));
        assert_eq!(node.numeric_value(), Some(25.0));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(100.0));
        assert_eq!(
            node.orientation(),
            Some(gpui::accesskit::Orientation::Horizontal)
        );
    }

    #[test]
    fn accessibility_requires_an_explicit_visible_role() {
        let key = NodeKey::new(171, 4);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("button".into()),
        )
        .unwrap();

        let materialize = |tree: &ProjectionTree| {
            ProjectedElement::from_tree(
                tree.snapshot(),
                key,
                ProjectedElementContext::default(),
                false,
            )
            .unwrap()
        };
        assert_eq!(
            materialize(&tree).a11y_role(),
            None,
            "the Rust projection must not infer web semantics from a tag"
        );

        tree.update_attribute(key, "role".into(), "button".into())
            .unwrap();
        assert_eq!(
            materialize(&tree).a11y_role(),
            Some(gpui::accesskit::Role::Button)
        );

        tree.update_attribute(key, "aria-hidden".into(), "true".into())
            .unwrap();
        assert_eq!(materialize(&tree).a11y_role(), None);
    }

    #[test]
    fn protocol_translation_moves_gpui_paint_without_changing_layout_style() {
        let key = NodeKey::new(18, 4);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("view".into()),
        )
        .unwrap();
        tree.update_transform(key, [1.0, 0.0, 0.0, 1.0, 14.0, -6.0])
            .unwrap();

        let element = ProjectedElement::from_tree(
            tree.snapshot(),
            key,
            ProjectedElementContext::default(),
            false,
        )
        .unwrap();
        assert_eq!(element.translation(), point(px(14.0), px(-6.0)));
        assert_eq!(element.style.size.width, Style::default().size.width);
        assert_eq!(element.style.size.height, Style::default().size.height);
    }

    #[test]
    fn text_controls_project_value_then_placeholder_without_web_dom_defaults() {
        let key = NodeKey::new(19, 4);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("input".into()),
        )
        .unwrap();
        tree.update_attribute(key, "placeholder".into(), "Search".into())
            .unwrap();
        assert_eq!(
            projected_text(&tree.snapshot(), tree.node(key).unwrap()),
            Some("Search".into())
        );

        tree.update_attribute(key, "value".into(), "typed".into())
            .unwrap();
        assert_eq!(
            projected_text(&tree.snapshot(), tree.node(key).unwrap()),
            Some("typed".into())
        );
    }

    #[test]
    fn aggregate_direct_text_is_one_gpui_text_run() {
        let parent = NodeKey::new(29, 1);
        let first = NodeKey::new(30, 1);
        let second = NodeKey::new(31, 1);
        let mut tree = ProjectionTree::default();
        tree.insert(
            parent,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("text".into()),
        )
        .unwrap();
        tree.insert(
            first,
            Some(parent),
            0,
            Style::default(),
            Some("Hello ".into()),
            crate::ProjectedNodeKind::Text,
        )
        .unwrap();
        tree.insert(
            second,
            Some(parent),
            1,
            Style::default(),
            Some("GPUI".into()),
            crate::ProjectedNodeKind::Text,
        )
        .unwrap();
        tree.update_text_behavior(parent, TEXT_BEHAVIOR_AGGREGATE_DIRECT)
            .unwrap();

        assert_eq!(
            projected_text(&tree.snapshot(), tree.node(parent).unwrap()),
            Some("Hello GPUI".into())
        );
        let element = ProjectedElement::from_tree(
            tree.snapshot(),
            parent,
            ProjectedElementContext::default(),
            false,
        )
        .unwrap();
        assert_eq!(element.children.len(), 1);
    }

    #[test]
    fn native_factories_receive_full_generational_keys_once_per_retained_node() {
        let old = NodeKey::new(27, 1);
        let recreated = NodeKey::new(27, 2);
        let mut tree = ProjectionTree::default();
        tree.insert(
            NodeKey::ROOT,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Root,
        )
        .unwrap();
        for (index, key) in [old, recreated].into_iter().enumerate() {
            tree.insert(
                key,
                Some(NodeKey::ROOT),
                index,
                Style::default(),
                None,
                crate::ProjectedNodeKind::Element("input".into()),
            )
            .unwrap();
        }
        let seen = Rc::new(RefCell::new(Vec::new()));
        let captured = seen.clone();
        let factory: ProjectedNativeElementFactory = Rc::new(move |key| {
            captured.borrow_mut().push(key);
            None
        });

        ProjectedElement::from_tree(
            tree.snapshot(),
            NodeKey::ROOT,
            ProjectedElementContext {
                native: Some(factory),
                ..Default::default()
            },
            false,
        )
        .unwrap();

        assert_eq!(*seen.borrow(), [NodeKey::ROOT, old, recreated]);
    }

    #[test]
    fn native_children_own_pointer_hit_testing() {
        let key = NodeKey::new(28, 1);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("slider".into()),
        )
        .unwrap();
        tree.update_interaction_policy(key, Some(0), false, false)
            .unwrap();
        tree.add_event_listener(key, wabou_protocol::event::CLICK)
            .unwrap();

        let native: ProjectedNativeElementFactory =
            Rc::new(move |candidate| (candidate == key).then(|| div().into_any_element()));
        let element = ProjectedElement::from_tree(
            tree.snapshot(),
            key,
            ProjectedElementContext {
                native: Some(native),
                input: Some(Rc::new(|_, _| {})),
                ..Default::default()
            },
            false,
        )
        .unwrap();

        assert!(element.input.is_some());
        assert!(
            !element.hit_testable,
            "the projected wrapper must not intercept gestures owned by its native child"
        );
    }

    #[test]
    fn blocked_interaction_policy_removes_native_input_from_the_subtree() {
        let child = NodeKey::new(27, 1);
        let mut tree = ProjectionTree::default();
        tree.insert(
            NodeKey::ROOT,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Root,
        )
        .unwrap();
        tree.insert(
            child,
            Some(NodeKey::ROOT),
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("button".into()),
        )
        .unwrap();
        tree.update_interaction_policy(NodeKey::ROOT, None, true, false)
            .unwrap();
        let input: ProjectedInputSink = Rc::new(|_, _| {});

        let root = ProjectedElement::from_tree(
            tree.snapshot(),
            NodeKey::ROOT,
            ProjectedElementContext {
                input: Some(input.clone()),
                ..Default::default()
            },
            false,
        )
        .unwrap();

        assert!(root.input.is_none());
    }

    #[test]
    fn pointer_events_none_skips_only_the_exact_hit_target() {
        let child_key = NodeKey::new(28, 1);
        let mut tree = ProjectionTree::default();
        tree.insert(
            NodeKey::ROOT,
            None,
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Root,
        )
        .unwrap();
        tree.insert(
            child_key,
            Some(NodeKey::ROOT),
            0,
            Style::default(),
            None,
            crate::ProjectedNodeKind::Element("button".into()),
        )
        .unwrap();
        tree.update_pointer_events(NodeKey::ROOT, false).unwrap();
        let input: ProjectedInputSink = Rc::new(|_, _| {});

        let root = ProjectedElement::from_tree(
            tree.snapshot(),
            NodeKey::ROOT,
            ProjectedElementContext {
                input: Some(input.clone()),
                ..Default::default()
            },
            false,
        )
        .unwrap();

        assert!(root.input.is_none());
        let child = ProjectedElement::from_tree(
            tree.snapshot(),
            child_key,
            ProjectedElementContext {
                input: Some(input),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        assert!(child.input.is_some());
        assert!(
            !child.hit_testable,
            "a visual descendant without pointer behavior must not steal its ancestor's hit target"
        );

        tree.add_event_listener(child_key, wabou_protocol::event::CLICK)
            .unwrap();
        let child = ProjectedElement::from_tree(
            tree.snapshot(),
            child_key,
            ProjectedElementContext {
                input: Some(Rc::new(|_, _| {})),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        assert!(child.hit_testable);
    }

    #[gpui::test]
    fn visual_descendants_do_not_replace_the_interactive_pointer_target(cx: &mut TestAppContext) {
        struct PointerHost {
            tree: ProjectionSnapshot,
            input: ProjectedInputSink,
        }
        impl gpui::Render for PointerHost {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let projected = ProjectedElement::from_tree(
                    self.tree.clone(),
                    NodeKey::new(80, 1),
                    ProjectedElementContext {
                        input: Some(self.input.clone()),
                        ..Default::default()
                    },
                    false,
                )
                .unwrap();
                div()
                    .size_full()
                    .child(gpui_base::TextSelectionLayer)
                    .child(projected)
            }
        }

        let button = NodeKey::new(80, 1);
        let label = NodeKey::new(81, 1);
        let mut button_style = Style::default();
        button_style.size.width = px(160.0).into();
        button_style.size.height = px(48.0).into();
        let mut tree = ProjectionTree::default();
        tree.insert(
            button,
            None,
            0,
            button_style,
            None,
            ProjectedNodeKind::Element("button".into()),
        )
        .unwrap();
        tree.insert(
            label,
            Some(button),
            0,
            Style::default(),
            Some("Interactive label".into()),
            ProjectedNodeKind::Text,
        )
        .unwrap();
        tree.add_event_listener(button, wabou_protocol::event::CLICK)
            .unwrap();
        tree.update_text_selection(button, Some(TextSelectionPolicy::None))
            .unwrap();

        let events = Rc::new(RefCell::new(Vec::new()));
        let observed = events.clone();
        let input: ProjectedInputSink = Rc::new(move |event, _| {
            if let ProjectedInputEvent::Pointer(event) = event {
                observed.borrow_mut().push(event);
            }
        });
        let (_view, cx) = cx.add_window_view(move |_, _| PointerHost {
            tree: tree.snapshot(),
            input,
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });

        let inside_label = point(px(24.0), px(20.0));
        cx.simulate_mouse_move(inside_label, None, Modifiers::default());
        cx.simulate_mouse_down(inside_label, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| {
            assert!(
                gpui_base::GlobalState::is_text_selection_suppressed(cx),
                "a select-none control must own its press before the window selection layer"
            );
        });
        cx.simulate_mouse_up(inside_label, MouseButton::Left, Modifiers::default());

        let events = events.borrow();
        assert_eq!(events.len(), 3);
        assert!(events.iter().all(|event| event.target == button));
        assert_eq!(events[0].phase, ProjectedPointerPhase::Move);
        assert_eq!(events[1].phase, ProjectedPointerPhase::Down);
        assert_eq!(events[2].phase, ProjectedPointerPhase::Up);
    }

    #[gpui::test]
    fn inline_svg_records_gpui_paint_success_and_failure(cx: &mut TestAppContext) {
        struct LayoutHost;
        impl gpui::Render for LayoutHost {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                div()
            }
        }

        let key = NodeKey::new(70, 1);
        let mut style = Style::default();
        style.size.width = px(24.0).into();
        style.size.height = px(24.0).into();
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            style,
            None,
            crate::ProjectedNodeKind::Element("svg".into()),
        )
        .unwrap();
        tree.update_svg_source(
            key,
            Some(std::sync::Arc::from(
                br#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 12h16"/></svg>"#
                    .as_slice(),
            )),
        )
        .unwrap();
        let states = ProjectedGraphicPaintStates::default();
        let (_host, window) = cx.add_window_view(|_, _| LayoutHost);
        window.draw(
            point(px(0.0), px(0.0)),
            gpui::size(px(24.0), px(24.0)),
            |_, _| {
                ProjectedElement::from_tree(
                    tree.snapshot(),
                    key,
                    ProjectedElementContext {
                        graphic_paint_states: Some(states.clone()),
                        ..Default::default()
                    },
                    false,
                )
                .unwrap()
            },
        );

        assert!(matches!(
            states.borrow().get(&key),
            Some(ProjectedGraphicPaintState::SvgPainted { bounds, color })
                if bounds.size == gpui::size(px(24.0), px(24.0))
                    && color.alpha > 0.0
        ));

        tree.update_svg_source(key, Some(std::sync::Arc::from(b"not svg".as_slice())))
            .unwrap();
        let (_host, invalid_window) = cx.add_window_view(|_, _| LayoutHost);
        invalid_window.draw(
            point(px(0.0), px(0.0)),
            gpui::size(px(24.0), px(24.0)),
            |_, _| {
                ProjectedElement::from_tree(
                    tree.snapshot(),
                    key,
                    ProjectedElementContext {
                        graphic_paint_states: Some(states.clone()),
                        ..Default::default()
                    },
                    false,
                )
                .unwrap()
            },
        );
        assert!(matches!(
            states.borrow().get(&key),
            Some(ProjectedGraphicPaintState::SvgFailed { message })
                if !message.is_empty()
        ));
    }

    #[gpui::test]
    fn prepaint_publishes_actual_gpui_layout_bounds(cx: &mut TestAppContext) {
        struct LayoutHost;
        impl gpui::Render for LayoutHost {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                div()
            }
        }

        let key = NodeKey::new(29, 3);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            Style::default(),
            Some("measured".into()),
            crate::ProjectedNodeKind::Text,
        )
        .unwrap();
        let bounds = ProjectedLayoutBounds::default();
        let observed = bounds.clone();
        let element = ProjectedElement::from_tree(
            tree.snapshot(),
            key,
            ProjectedElementContext {
                layout_bounds: Some(bounds),
                ..Default::default()
            },
            false,
        )
        .unwrap();

        let (_host, window) = cx.add_window_view(|_, _| LayoutHost);
        window.draw(
            point(px(7.0), px(11.0)),
            gpui::size(px(320.0), px(200.0)),
            |_, _| element,
        );

        let bounds = observed.borrow()[&key];
        assert_eq!(bounds.origin, point(px(7.0), px(11.0)));
        assert!(bounds.size.width > px(0.0));
        assert!(bounds.size.height > px(0.0));
    }

    #[gpui::test]
    fn display_none_skips_text_measure_and_prepaint_together(cx: &mut TestAppContext) {
        struct LayoutHost;
        impl gpui::Render for LayoutHost {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                div()
            }
        }

        let hidden = NodeKey::new(30, 3);
        let text = NodeKey::new(31, 3);
        let mut hidden_style = Style::default();
        hidden_style.display = gpui::Display::None;
        let mut tree = ProjectionTree::default();
        tree.insert(
            hidden,
            None,
            0,
            hidden_style,
            None,
            crate::ProjectedNodeKind::Element("view".into()),
        )
        .unwrap();
        tree.insert(
            text,
            Some(hidden),
            0,
            Style::default(),
            Some("hidden text must not be prepainted".into()),
            crate::ProjectedNodeKind::Text,
        )
        .unwrap();
        let element = ProjectedElement::from_tree(
            tree.snapshot(),
            hidden,
            ProjectedElementContext::default(),
            false,
        )
        .unwrap();

        let (_host, window) = cx.add_window_view(|_, _| LayoutHost);
        window.draw(
            point(px(0.0), px(0.0)),
            gpui::size(px(320.0), px(200.0)),
            |_, _| element,
        );
    }

    #[gpui::test]
    fn inherited_text_style_is_active_while_gpui_shapes_child_layout(cx: &mut TestAppContext) {
        struct LayoutHost;
        impl gpui::Render for LayoutHost {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                div()
            }
        }

        struct TextStyleProbe(Rc<RefCell<Option<gpui::Hsla>>>);
        impl IntoElement for TextStyleProbe {
            type Element = Self;

            fn into_element(self) -> Self::Element {
                self
            }
        }
        impl Element for TextStyleProbe {
            type RequestLayoutState = ();
            type PrepaintState = ();

            fn id(&self) -> Option<ElementId> {
                None
            }

            fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
                None
            }

            fn request_layout(
                &mut self,
                _id: Option<&GlobalElementId>,
                _inspector_id: Option<&InspectorElementId>,
                window: &mut Window,
                cx: &mut App,
            ) -> (LayoutId, Self::RequestLayoutState) {
                *self.0.borrow_mut() = Some(window.text_style().color);
                (window.request_layout(Style::default(), [], cx), ())
            }

            fn prepaint(
                &mut self,
                _id: Option<&GlobalElementId>,
                _inspector_id: Option<&InspectorElementId>,
                _bounds: Bounds<Pixels>,
                _request_layout: &mut Self::RequestLayoutState,
                _window: &mut Window,
                _cx: &mut App,
            ) {
            }

            fn paint(
                &mut self,
                _id: Option<&GlobalElementId>,
                _inspector_id: Option<&InspectorElementId>,
                _bounds: Bounds<Pixels>,
                _request_layout: &mut Self::RequestLayoutState,
                _prepaint: &mut Self::PrepaintState,
                _window: &mut Window,
                _cx: &mut App,
            ) {
            }
        }

        let key = NodeKey::new(41, 1);
        let expected = gpui::rgb_to_hsla(gpui::rgba(0xf2f4_f7ff));
        let mut style = Style::default();
        style.text.color = Some(expected);
        let mut tree = ProjectionTree::default();
        tree.insert(
            key,
            None,
            0,
            style,
            None,
            crate::ProjectedNodeKind::Element("view".into()),
        )
        .unwrap();
        let mut element = ProjectedElement::from_tree(
            tree.snapshot(),
            key,
            ProjectedElementContext::default(),
            false,
        )
        .unwrap();
        let observed = Rc::new(RefCell::new(None));
        element
            .children
            .push(TextStyleProbe(observed.clone()).into_any_element());

        let (_host, window) = cx.add_window_view(|_, _| LayoutHost);
        window.draw(
            point(px(0.0), px(0.0)),
            gpui::size(px(120.0), px(40.0)),
            |_, _| element,
        );

        assert_eq!(*observed.borrow(), Some(expected));
    }

    #[test]
    fn wheel_projection_preserves_native_units_phase_and_target() {
        let key = NodeKey::new(23, 5);
        let event = ScrollWheelEvent {
            position: point(px(40.0), px(80.0)),
            delta: ScrollDelta::Pixels(point(px(-2.5), px(12.0))),
            modifiers: Modifiers {
                shift: true,
                control: false,
                alt: true,
                platform: false,
                function: false,
            },
            touch_phase: TouchPhase::Started,
        };

        assert_eq!(
            wheel_event(
                key,
                &event,
                Bounds {
                    origin: point(px(10.0), px(30.0)),
                    size: gpui::size(px(100.0), px(100.0)),
                },
            ),
            ProjectedWheelEvent {
                target: key,
                x: 40.0,
                y: 80.0,
                local_x: 30.0,
                local_y: 50.0,
                delta_x: -2.5,
                delta_y: 12.0,
                precise: true,
                phase: ProjectedWheelPhase::Started,
                shift: true,
                control: false,
                alt: true,
                platform: false,
            }
        );
    }

    #[test]
    fn retained_scroll_state_clamps_absolute_relative_and_partial_updates() {
        let handle = ProjectedScrollHandle::default();
        handle.set_max_offset(point(px(100.0), px(200.0)));

        assert!(handle.scroll_to(f32::NAN, 80.0));
        assert_eq!(handle.position(), point(px(0.0), px(80.0)));
        assert!(handle.scroll_by(30.0, 150.0));
        assert_eq!(handle.position(), point(px(30.0), px(200.0)));
        assert!(!handle.scroll_by(0.0, 20.0));
        assert!(handle.scroll_to(500.0, f32::NAN));
        assert_eq!(handle.position(), point(px(100.0), px(200.0)));

        handle.set_max_offset(point(px(40.0), px(60.0)));
        assert_eq!(handle.position(), point(px(40.0), px(60.0)));
    }

    #[gpui::test]
    fn retained_scroll_moves_children_without_rebuilding_the_tree(cx: &mut TestAppContext) {
        struct LayoutHost;
        impl gpui::Render for LayoutHost {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                div()
            }
        }

        let parent = NodeKey::new(40, 1);
        let child = NodeKey::new(41, 1);
        let mut parent_style = Style::default();
        parent_style.size.width = px(100.0).into();
        parent_style.size.height = px(100.0).into();
        parent_style.overflow.y = Overflow::Scroll;
        let mut child_style = Style::default();
        child_style.size.width = px(100.0).into();
        child_style.size.height = px(240.0).into();
        child_style.flex_shrink = 0.0;
        let mut tree = ProjectionTree::default();
        tree.insert(
            parent,
            None,
            0,
            parent_style,
            None,
            crate::ProjectedNodeKind::Element("view".into()),
        )
        .unwrap();
        tree.insert(
            child,
            Some(parent),
            0,
            child_style,
            None,
            crate::ProjectedNodeKind::Element("view".into()),
        )
        .unwrap();
        let handle = ProjectedScrollHandle::default();
        let scroll_handles = Rc::new(BTreeMap::from([(parent, handle.clone())]));
        let bounds = ProjectedLayoutBounds::default();
        let (_host, window) = cx.add_window_view(|_, _| LayoutHost);

        let materialize = || {
            ProjectedElement::from_tree(
                tree.snapshot(),
                parent,
                ProjectedElementContext {
                    layout_bounds: Some(bounds.clone()),
                    scroll_handles: Some(scroll_handles.clone()),
                    ..Default::default()
                },
                false,
            )
            .unwrap()
        };
        window.draw(
            point(px(0.0), px(0.0)),
            gpui::size(px(100.0), px(100.0)),
            |_, _| materialize(),
        );
        assert_eq!(bounds.borrow()[&child].origin.y, px(0.0));

        assert!(handle.scroll_to(f32::NAN, 50.0));
        let (_host, second_window) = cx.add_window_view(|_, _| LayoutHost);
        second_window.draw(
            point(px(0.0), px(0.0)),
            gpui::size(px(100.0), px(100.0)),
            |_, _| materialize(),
        );
        assert_eq!(bounds.borrow()[&child].origin.y, px(-50.0));
    }

    #[gpui::test]
    fn uniform_virtual_list_materializes_only_gpui_visible_rows(cx: &mut TestAppContext) {
        struct LayoutHost {
            tree: ProjectionSnapshot,
            root: NodeKey,
            bounds: ProjectedLayoutBounds,
            list_handles: Rc<BTreeMap<NodeKey, UniformListScrollHandle>>,
        }
        impl gpui::Render for LayoutHost {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                ProjectedElement::from_tree(
                    self.tree.clone(),
                    self.root,
                    ProjectedElementContext {
                        layout_bounds: Some(self.bounds.clone()),
                        uniform_list_handles: Some(self.list_handles.clone()),
                        ..Default::default()
                    },
                    false,
                )
                .unwrap()
            }
        }

        let list = NodeKey::new(200, 1);
        let mut list_style = Style::default();
        list_style.size.width = px(100.0).into();
        list_style.size.height = px(100.0).into();
        let mut tree = ProjectionTree::default();
        tree.insert(
            list,
            None,
            0,
            list_style,
            None,
            ProjectedNodeKind::Element("virtual-list".into()),
        )
        .unwrap();
        for index in 0..100 {
            let mut row_style = Style::default();
            row_style.size.width = gpui::relative(1.0).into();
            row_style.size.height = px(20.0).into();
            row_style.flex_shrink = 0.0;
            tree.insert(
                NodeKey::new(201 + index, 1),
                Some(list),
                index as usize,
                row_style,
                Some(format!("row {index}").into()),
                ProjectedNodeKind::Element("view".into()),
            )
            .unwrap();
        }

        let bounds = ProjectedLayoutBounds::default();
        let list_handles = Rc::new(BTreeMap::from([(list, UniformListScrollHandle::default())]));
        let (_host, _window) = cx.add_window_view(|_, _| LayoutHost {
            tree: tree.snapshot(),
            root: list,
            bounds: bounds.clone(),
            list_handles,
        });

        let visible_rows = bounds.borrow().keys().filter(|key| **key != list).count();
        assert!(
            visible_rows > 0,
            "the native list must materialize visible rows"
        );
        assert!(
            visible_rows < 100,
            "the native list materialized all {visible_rows} retained rows"
        );
    }

    #[test]
    fn key_projection_keeps_layout_key_character_and_repeat_separate() {
        let event = key_event(
            ProjectedKeyPhase::Down,
            &Keystroke {
                modifiers: Modifiers {
                    shift: true,
                    control: false,
                    alt: false,
                    platform: false,
                    function: false,
                },
                key: "a".into(),
                key_char: Some("A".into()),
            },
            true,
        );

        assert_eq!(event.phase, ProjectedKeyPhase::Down);
        assert_eq!(event.key, "a");
        assert_eq!(event.key_char.as_deref(), Some("A"));
        assert!(event.repeat);
        assert!(event.shift);
    }

    struct InputHarness {
        tree: ProjectionTree,
        focus: FocusHandle,
        input: ProjectedInputSink,
    }

    impl gpui::Render for InputHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            self.tree
                .interactive_element(
                    NodeKey::ROOT,
                    self.input.clone(),
                    self.focus.clone(),
                    ProjectedTextInputState {
                        accepts_text: true,
                        ..Default::default()
                    },
                    None,
                )
                .expect("input projection")
        }
    }

    #[gpui::test]
    fn gpui_input_handler_commits_each_platform_character_once(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        let captured = events.clone();
        let input: ProjectedInputSink = Rc::new(move |event, _| {
            captured.borrow_mut().push(event);
        });
        let (_view, cx) = cx.add_window_view(move |window, cx| {
            let mut tree = ProjectionTree::default();
            tree.insert(
                NodeKey::ROOT,
                None,
                0,
                Style::default(),
                Some("input".into()),
                crate::ProjectedNodeKind::Root,
            )
            .unwrap();
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            InputHarness { tree, focus, input }
        });

        cx.simulate_input("日本");

        let events = events.borrow();
        let commits = events
            .iter()
            .filter_map(|event| match event {
                ProjectedInputEvent::Ime(crate::ProjectedImeEvent::Commit(text)) => {
                    Some(text.as_str())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(commits, ["日", "本"], "all projected events: {events:?}");
    }

    #[gpui::test]
    fn projected_text_uses_native_window_selection(cx: &mut TestAppContext) {
        struct SelectionHost {
            tree: ProjectionSnapshot,
            selection: crate::ProjectedTextSelection,
            bounds: ProjectedLayoutBounds,
            focus: FocusHandle,
        }

        impl gpui::Render for SelectionHost {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let selections = Rc::new(BTreeMap::from([(
                    NodeKey::new(2, 1),
                    self.selection.clone(),
                )]));
                let projected = ProjectedElement::from_tree(
                    self.tree.clone(),
                    NodeKey::ROOT,
                    ProjectedElementContext {
                        input: Some(Rc::new(|_, _| {})),
                        root_focus: Some(self.focus.clone()),
                        text_selections: Some(selections),
                        layout_bounds: Some(self.bounds.clone()),
                        ..Default::default()
                    },
                    false,
                )
                .unwrap();
                div()
                    .size_full()
                    .child(projected)
                    .child(gpui_base::TextSelectionLayer)
            }
        }

        let mut projection = crate::GpuiProjection::new();
        let atoms = wabou_protocol::AtomPool::default();
        projection
            .apply_ops(
                &wabou_protocol::Frame {
                    seq: 1,
                    ops: vec![
                        wabou_protocol::Op::CreateText {
                            id: NodeKey::new(2, 1),
                            text: "selectable sentence",
                        },
                        wabou_protocol::Op::AppendChild {
                            parent: NodeKey::ROOT,
                            child: NodeKey::new(2, 1),
                        },
                    ],
                },
                &atoms,
                |_| None,
            )
            .unwrap();
        let _ = projection.finish_frame();
        let tree = projection.tree().snapshot();
        let bounds = ProjectedLayoutBounds::default();
        let observed_bounds = bounds.clone();
        let (_view, cx) = cx.add_window_view(move |window, cx| {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            SelectionHost {
                tree,
                selection: crate::ProjectedTextSelection::new(
                    gpui_base::TextSelectionHandle::new("selectable sentence", cx),
                    0,
                    false,
                ),
                bounds,
                focus,
            }
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let text_bounds = observed_bounds.borrow()[&NodeKey::new(2, 1)];
        let start = point(text_bounds.left() + px(2.0), text_bounds.center().y);
        let end = point(text_bounds.right() - px(2.0), text_bounds.center().y);
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        cx.update(|window, cx| {
            assert!(!gpui_base::TextSelection::selected_text(window, cx).is_empty());
        });
        cx.simulate_keystrokes("ctrl-c");
        assert!(
            cx.read_from_clipboard()
                .and_then(|item| item.text())
                .is_some_and(|text| !text.is_empty())
        );
    }
}
