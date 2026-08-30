use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Element, ElementId, GlobalElementId, Hitbox,
    HitboxBehavior, InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Visibility, Window, div, prelude::*,
};

use crate::{
    GpuiNodeKeyExt, NodeKey, ProjectedInputSink, ProjectedPointerButton, ProjectedPointerEvent,
    ProjectedPointerPhase, ProjectionError, ProjectionTree,
};

/// A lightweight GPUI element generated from one Wabou retained node.
///
/// GPUI drops element objects after each frame. Stable state survives through
/// [`Element::id`], which is derived losslessly from Wabou's generational key.
pub struct ProjectedElement {
    key: NodeKey,
    style: gpui::Style,
    children: Vec<AnyElement>,
    input: Option<ProjectedInputSink>,
}

impl ProjectedElement {
    pub(crate) fn from_tree(
        tree: &ProjectionTree,
        key: NodeKey,
        input: Option<ProjectedInputSink>,
    ) -> Result<Self, ProjectionError> {
        let node = tree.node(key).ok_or(ProjectionError::MissingNode(key))?;
        let mut children =
            Vec::with_capacity(node.children.len() + usize::from(node.text.is_some()));
        if let Some(text) = &node.text {
            children.push(div().child(text.clone()).into_any_element());
        }
        for child in &node.children {
            children.push(Self::from_tree(tree, *child, input.clone())?.into_any_element());
        }
        Ok(Self {
            key,
            style: node.style.clone(),
            children,
            input,
        })
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
    button: Option<MouseButton>,
    modifiers: gpui::Modifiers,
) -> ProjectedPointerEvent {
    ProjectedPointerEvent {
        target: key,
        phase,
        x: position.x.into(),
        y: position.y.into(),
        button: button.map(pointer_button),
        shift: modifiers.shift,
        control: modifiers.control,
        alt: modifiers.alt,
        platform: modifiers.platform,
    }
}

impl IntoElement for ProjectedElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ProjectedElement {
    type RequestLayoutState = ();
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        Some(self.key.gpui_element_id())
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
        let child_layouts = self
            .children
            .iter_mut()
            .map(|child| child.request_layout(window, cx))
            .collect::<Vec<_>>();
        (
            window.request_layout(self.style.clone(), child_layouts, cx),
            (),
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let text_style = self.style.text_style().cloned();
        let overflow_mask = self.style.overflow_mask(bounds, window.rem_size());
        let hitbox = self
            .input
            .as_ref()
            .map(|_| window.insert_hitbox(bounds, HitboxBehavior::Normal));
        window.with_text_style(text_style, |window| {
            window.with_content_mask(overflow_mask, |window| {
                for child in &mut self.children {
                    child.prepaint(window, cx);
                }
            });
        });
        hitbox
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.style.visibility == Visibility::Hidden {
            return;
        }

        if let (Some(input), Some(hitbox)) = (&self.input, hitbox.as_ref()) {
            let down_input = input.clone();
            let down_hitbox = hitbox.clone();
            let key = self.key;
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && down_hitbox.is_hovered(window) {
                    down_input(
                        pointer_event(
                            key,
                            ProjectedPointerPhase::Down,
                            event.position,
                            Some(event.button),
                            event.modifiers,
                        ),
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
                        pointer_event(
                            key,
                            ProjectedPointerPhase::Up,
                            event.position,
                            Some(event.button),
                            event.modifiers,
                        ),
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
                        pointer_event(
                            key,
                            ProjectedPointerPhase::Move,
                            event.position,
                            event.pressed_button,
                            event.modifiers,
                        ),
                        cx,
                    );
                    cx.stop_propagation();
                }
            });
        }

        let text_style = self.style.text_style().cloned();
        let overflow_mask = self.style.overflow_mask(bounds, window.rem_size());
        self.style.paint(bounds, window, cx, |window, cx| {
            window.with_text_style(text_style, |window| {
                window.with_content_mask(overflow_mask, |window| {
                    for child in &mut self.children {
                        child.paint(window, cx);
                    }
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Style;

    #[test]
    fn generated_element_uses_the_retained_generational_identity() {
        let key = NodeKey::new(17, 4);
        let mut tree = ProjectionTree::default();
        tree.insert(key, None, 0, Style::default(), Some("hello".into()))
            .unwrap();

        let element = ProjectedElement::from_tree(&tree, key, None).unwrap();
        assert_eq!(element.id(), Some(key.gpui_element_id()));
    }
}
