use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Element, ElementId, FocusHandle, GlobalElementId,
    Hitbox, HitboxBehavior, InspectorElementId, IntoElement, KeyDownEvent, KeyUpEvent, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, ScrollDelta,
    ScrollWheelEvent, TouchPhase, Visibility, Window, div, prelude::*,
};

use crate::{
    GpuiNodeKeyExt, NodeKey, ProjectedInputEvent, ProjectedInputSink, ProjectedKeyEvent,
    ProjectedKeyPhase, ProjectedNode, ProjectedNodeKind, ProjectedPointerButton,
    ProjectedPointerEvent, ProjectedPointerPhase, ProjectedTextInputState, ProjectedWheelEvent,
    ProjectedWheelPhase, ProjectionError, ProjectionTree,
};

/// Produces a GPUI-owned native control for a retained Wabou node.
///
/// The callback is evaluated once while materializing a frame. It lets the
/// runtime preserve platform control state in `Entity<T>` without moving that
/// state into the lightweight projection cache.
pub type ProjectedNativeElementFactory = Rc<dyn Fn(NodeKey) -> Option<AnyElement>>;

pub(crate) type ProjectedLayoutBounds = Rc<RefCell<BTreeMap<NodeKey, Bounds<Pixels>>>>;

#[derive(Clone, Default)]
pub(crate) struct ProjectedElementContext {
    pub(crate) input: Option<ProjectedInputSink>,
    pub(crate) root_focus: Option<FocusHandle>,
    pub(crate) text_input: Option<ProjectedTextInputState>,
    pub(crate) native: Option<ProjectedNativeElementFactory>,
    pub(crate) layout_bounds: Option<ProjectedLayoutBounds>,
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
    root_focus: Option<FocusHandle>,
    text_input: Option<ProjectedTextInputState>,
    layout_bounds: Option<ProjectedLayoutBounds>,
    transform: [f32; 6],
}

pub struct ProjectedPrepaintState {
    hitbox: Option<Hitbox>,
    paint_bounds: Bounds<Pixels>,
}

impl ProjectedElement {
    pub(crate) fn from_tree(
        tree: &ProjectionTree,
        key: NodeKey,
        context: ProjectedElementContext,
        ancestor_blocked: bool,
    ) -> Result<Self, ProjectionError> {
        let node = tree.node(key).ok_or(ProjectionError::MissingNode(key))?;
        let interaction_blocked = ancestor_blocked || node.interaction_blocked;
        let native_child = context.native.as_ref().and_then(|factory| factory(key));
        let mut children =
            Vec::with_capacity(node.children.len() + usize::from(node.text.is_some()));
        if let Some(native_child) = native_child {
            children.push(native_child);
        } else if let Some(text) = projected_text(node) {
            children.push(div().child(text.clone()).into_any_element());
        }
        if let Some(image) = &node.image {
            children.push(gpui::img(image.clone()).size_full().into_any_element());
        }
        for child in &node.children {
            let projected =
                Self::from_tree(tree, *child, context.for_child(), interaction_blocked)?;
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
        Ok(Self {
            key,
            style: node.style.clone(),
            children,
            input: (!interaction_blocked && node.pointer_events)
                .then_some(context.input)
                .flatten(),
            root_focus: context.root_focus,
            text_input: context.text_input,
            layout_bounds: context.layout_bounds,
            transform: node.transform,
        })
    }

    fn translation(&self) -> gpui::Point<Pixels> {
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
}

fn projected_text(node: &ProjectedNode) -> Option<&gpui::SharedString> {
    node.text.as_ref().or_else(|| {
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
    type RequestLayoutState = ();
    type PrepaintState = ProjectedPrepaintState;

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
        if let Some(layout_bounds) = &self.layout_bounds {
            layout_bounds.borrow_mut().insert(self.key, bounds);
        }
        let translation = self.translation();
        let paint_bounds = Bounds {
            origin: bounds.origin + translation,
            size: bounds.size,
        };
        let text_style = self.style.text_style().cloned();
        let overflow_mask = self.style.overflow_mask(paint_bounds, window.rem_size());
        if let Some(focus) = &self.root_focus {
            window.set_focus_handle(focus, cx);
        }
        let hitbox = self
            .input
            .as_ref()
            .map(|_| window.insert_hitbox(paint_bounds, HitboxBehavior::Normal));
        window.with_text_style(text_style, |window| {
            window.with_content_mask(overflow_mask, |window| {
                window.with_element_offset(translation, |window| {
                    for child in &mut self.children {
                        child.prepaint(window, cx);
                    }
                });
            });
        });
        ProjectedPrepaintState {
            hitbox,
            paint_bounds,
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
        if self.style.visibility == Visibility::Hidden {
            return;
        }

        let bounds = prepaint.paint_bounds;
        if let (Some(input), Some(hitbox)) = (&self.input, prepaint.hitbox.as_ref()) {
            let down_input = input.clone();
            let down_hitbox = hitbox.clone();
            let key = self.key;
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && down_hitbox.is_hovered(window) {
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

            let wheel_input = input.clone();
            let wheel_hitbox = hitbox.clone();
            let key = self.key;
            window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && wheel_hitbox.should_handle_scroll(window) {
                    wheel_input(
                        ProjectedInputEvent::Wheel(wheel_event(key, event, bounds)),
                        cx,
                    );
                    cx.stop_propagation();
                }
            });
        }

        if let (Some(input), Some(_focus)) = (&self.input, &self.root_focus) {
            let down_input = input.clone();
            window.on_key_event(move |event: &KeyDownEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble {
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
    use std::{cell::RefCell, rc::Rc};

    use gpui::{Context, Keystroke, Modifiers, Style, TestAppContext, point, px};

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

        let element =
            ProjectedElement::from_tree(&tree, key, ProjectedElementContext::default(), false)
                .unwrap();
        assert_eq!(element.id(), Some(key.gpui_element_id()));
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

        let element =
            ProjectedElement::from_tree(&tree, key, ProjectedElementContext::default(), false)
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
            projected_text(tree.node(key).unwrap()).map(AsRef::as_ref),
            Some("Search")
        );

        tree.update_attribute(key, "value".into(), "typed".into())
            .unwrap();
        assert_eq!(
            projected_text(tree.node(key).unwrap()).map(AsRef::as_ref),
            Some("typed")
        );
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
            &tree,
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
            &tree,
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
        let child = NodeKey::new(28, 1);
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
        tree.update_pointer_events(NodeKey::ROOT, false).unwrap();
        let input: ProjectedInputSink = Rc::new(|_, _| {});

        let root = ProjectedElement::from_tree(
            &tree,
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
            &tree,
            child,
            ProjectedElementContext {
                input: Some(input),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        assert!(child.input.is_some());
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
            &tree,
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
}
