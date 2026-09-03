use std::{collections::BTreeMap, ops::Range, rc::Rc};

use gpui::{
    AnyElement, App, BorderStyle, Bounds, Corners, CursorStyle, Edges, Element, ElementId,
    GlobalElementId, Hitbox, InspectorElementId, IntoElement, LayoutId, PaintQuad, Pixels, Point,
    SharedString, StyledText, Window, transparent_black,
};
use gpui_base::{TextSelectionHandle, TextSelectionRegistration, TextSelectionRun};

use crate::NodeKey;

/// Stable native selection participant retained for one projected text node.
#[derive(Clone)]
pub struct ProjectedTextSelection {
    pub(crate) handle: TextSelectionHandle,
    pub(crate) document_order: u64,
    pub(crate) select_all: bool,
}

impl ProjectedTextSelection {
    #[must_use]
    pub fn new(handle: TextSelectionHandle, document_order: u64, select_all: bool) -> Self {
        Self {
            handle,
            document_order,
            select_all,
        }
    }
}

pub(crate) type ProjectedTextSelections = Rc<BTreeMap<NodeKey, ProjectedTextSelection>>;

pub(crate) fn selectable_text_element(
    selection: ProjectedTextSelection,
    text: SharedString,
    styled_text: StyledText,
) -> AnyElement {
    SelectableTextElement {
        selection,
        styled_text,
        text,
    }
    .into_any_element()
}

struct SelectableTextElement {
    selection: ProjectedTextSelection,
    text: SharedString,
    styled_text: StyledText,
}

fn selection_quad_bounds(
    start: Point<Pixels>,
    end: Point<Pixels>,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
) -> Vec<Bounds<Pixels>> {
    if start.y == end.y {
        return vec![Bounds::from_corners(
            start,
            Point::new(end.x, end.y + line_height),
        )];
    }
    let mut quads = vec![Bounds::from_corners(
        start,
        Point::new(bounds.right(), start.y + line_height),
    )];
    if end.y > start.y + line_height {
        quads.push(Bounds::from_corners(
            Point::new(bounds.left(), start.y + line_height),
            Point::new(bounds.right(), end.y),
        ));
    }
    quads.push(Bounds::from_corners(
        Point::new(bounds.left(), end.y),
        Point::new(end.x, end.y + line_height),
    ));
    quads
}

fn paint_selection(layout: &gpui::TextLayout, range: Range<usize>, window: &mut Window) {
    let (Some(start), Some(end)) = (
        layout.position_for_index(range.start),
        layout.position_for_index(range.end),
    ) else {
        return;
    };
    // A conventional native-blue highlight remains legible in both Wabou
    // themes without coupling the shell adapter to application theme tokens.
    let color = gpui::hsla(0.58, 0.85, 0.62, 0.35);
    for bounds in selection_quad_bounds(start, end, layout.bounds(), layout.line_height()) {
        window.paint_quad(PaintQuad {
            bounds,
            background: color.into(),
            corner_radii: Corners::default(),
            border_widths: Edges::default(),
            border_color: transparent_black(),
            border_style: BorderStyle::default(),
        });
    }
}

impl IntoElement for SelectableTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SelectableTextElement {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.styled_text
            .request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.styled_text
            .prepaint(id, inspector_id, bounds, &mut (), window, cx);
        let hitbox = window.insert_hitbox(bounds, gpui::HitboxBehavior::Normal);
        self.selection.handle.register(
            TextSelectionRegistration::new(hitbox.clone(), bounds)
                .with_document_order(self.selection.document_order)
                .with_text_bounds(vec![bounds]),
            window,
            cx,
        );
        hitbox
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.set_cursor_style(CursorStyle::IBeam, hitbox);
        if self.selection.select_all && hitbox.is_hovered(window) {
            let selection = self.selection.handle.clone();
            window.on_mouse_event(move |event: &gpui::MouseDownEvent, phase, _, cx| {
                if phase.bubble() && event.button == gpui::MouseButton::Left {
                    selection.set_local_selection(true, cx);
                }
            });
        }
        let layout = self.styled_text.layout().clone();
        let projection = self.selection.handle.update_runs(
            &[TextSelectionRun::new(
                self.text.clone(),
                layout.clone(),
                bounds,
            )],
            cx,
        );
        let range = self
            .selection
            .handle
            .has_local_selection(cx)
            .then(|| 0..self.text.len())
            .or_else(|| projection.ranges().first().and_then(Clone::clone));
        if let Some(range) = range {
            paint_selection(&layout, range, window);
        }
        self.styled_text
            .paint(id, inspector_id, bounds, &mut (), &mut (), window, cx);
    }
}
