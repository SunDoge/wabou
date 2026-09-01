//! Application-defined GPUI elements mounted behind explicit Wabou tags.

use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use gpui::{AnyElement, AnyEntity, App, Context, Entity, SharedString, Window};
use wabou_shell_api::UiEvent;

use crate::NodeKey;

/// Owned event bridge for callbacks retained by a native GPUI widget.
///
/// The bridge keeps the exact generational Wabou node identity while avoiding
/// a borrow of the ephemeral [`NativeWidgetContext`]. GPUI callbacks can clone
/// this value and dispatch semantic events after the factory has returned.
#[derive(Clone)]
pub struct NativeWidgetEventSink {
    target: NodeKey,
    input: crate::ProjectedInputSink,
}

impl NativeWidgetEventSink {
    #[doc(hidden)]
    pub fn new(target: NodeKey, input: crate::ProjectedInputSink) -> Self {
        Self { target, input }
    }

    /// Dispatch one semantic activation through Wabou's normal batched event
    /// path. The native widget owns gesture recognition, so no synthetic
    /// pointer coordinates or duplicate hit target are introduced.
    pub fn activate(&self, cx: &mut App) {
        (self.input)(
            crate::ProjectedInputEvent::Activate {
                target: self.target,
            },
            cx,
        );
    }

    /// Dispatch a numeric control value through Wabou's typed change event.
    ///
    /// Sliders, color channels, and similar retained controls use this instead
    /// of serializing a custom JSON message or reconstructing pointer geometry
    /// in JavaScript.
    pub fn change_f64(&self, value: f64, cx: &mut App) {
        (self.input)(
            crate::ProjectedInputEvent::ValueChange {
                target: self.target,
                value,
            },
            cx,
        );
    }

    /// Commit a text value through Wabou's typed input event.
    ///
    /// Native editors use this after GPUI has applied keyboard or IME input.
    /// JavaScript receives the same controlled `input` event as a Wabou text
    /// primitive without needing a widget-specific JSON message.
    pub fn input_text(&self, value: impl Into<String>, cx: &mut App) {
        (self.input)(
            crate::ProjectedInputEvent::TextChange {
                target: self.target,
                value: value.into(),
            },
            cx,
        );
    }

    /// Report native focus ownership through Wabou's ordinary focus events.
    pub fn focus(&self, focused: bool, cx: &mut App) {
        (self.input)(
            crate::ProjectedInputEvent::FocusChange {
                target: self.target,
                focused,
            },
            cx,
        );
    }

    /// Report a text selection using JavaScript's UTF-16 offset convention.
    pub fn text_selection(&self, anchor: u32, head: u32, cx: &mut App) {
        (self.input)(
            crate::ProjectedInputEvent::TextSelectionChange {
                target: self.target,
                anchor,
                head,
            },
            cx,
        );
    }

    /// Report a submit gesture recognized by a retained native control.
    pub fn submit(&self, secondary: bool, shift: bool, cx: &mut App) {
        (self.input)(
            crate::ProjectedInputEvent::Submit {
                target: self.target,
                secondary,
                shift,
            },
            cx,
        );
    }
}

/// Immutable authored state supplied when materializing a native GPUI widget.
///
/// The node key is generational and remains stable across ordinary Solid
/// updates. Stateful widgets may use it as their GPUI element identity or as a
/// key into an application-owned entity registry.
pub struct NativeWidgetContext<'a> {
    key: NodeKey,
    attributes: &'a BTreeMap<SharedString, SharedString>,
    previous_attributes: Option<&'a BTreeMap<SharedString, SharedString>>,
    config: Option<&'a str>,
    entity: Option<&'a AnyEntity>,
    input: crate::ProjectedInputSink,
}

impl<'a> NativeWidgetContext<'a> {
    #[doc(hidden)]
    pub fn new(
        key: NodeKey,
        attributes: &'a BTreeMap<SharedString, SharedString>,
        config: Option<&'a str>,
        entity: Option<&'a AnyEntity>,
        input: crate::ProjectedInputSink,
    ) -> Self {
        Self {
            key,
            attributes,
            previous_attributes: None,
            config,
            entity,
            input,
        }
    }

    #[doc(hidden)]
    pub fn new_with_previous_attributes(
        key: NodeKey,
        attributes: &'a BTreeMap<SharedString, SharedString>,
        previous_attributes: Option<&'a BTreeMap<SharedString, SharedString>>,
        config: Option<&'a str>,
        entity: Option<&'a AnyEntity>,
        input: crate::ProjectedInputSink,
    ) -> Self {
        Self {
            key,
            attributes,
            previous_attributes,
            config,
            entity,
            input,
        }
    }

    /// Return the exact generational identity of the authored Solid node.
    #[must_use]
    pub fn key(&self) -> NodeKey {
        self.key
    }

    /// Read one authored attribute without allocating.
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(AsRef::as_ref)
    }

    /// Iterate over all authored attributes in deterministic key order.
    pub fn attributes(&self) -> impl Iterator<Item = (&str, &str)> {
        self.attributes
            .iter()
            .map(|(name, value)| (name.as_ref(), value.as_ref()))
    }

    /// Whether the authored attribute map changed since this retained widget
    /// was last materialized. Initial mounts always report a change.
    #[must_use]
    pub fn attributes_changed(&self) -> bool {
        self.previous_attributes != Some(self.attributes)
    }

    /// Return the validated JSON payload authored through `widgetConfig`.
    ///
    /// Wabou deliberately keeps this transport representation opaque here:
    /// application widgets can deserialize directly into their own DTO type.
    #[must_use]
    pub fn config_json(&self) -> Option<&str> {
        self.config
    }

    /// Recover the stable GPUI entity retained for this node, when its type
    /// matches the factory's state type.
    #[must_use]
    pub fn entity<T: 'static>(&self) -> Option<Entity<T>> {
        self.entity?.clone().downcast().ok()
    }

    /// Return an owned bridge suitable for GPUI's retained event callbacks.
    #[must_use]
    pub fn events(&self) -> NativeWidgetEventSink {
        NativeWidgetEventSink::new(self.key, self.input.clone())
    }

    /// Dispatch one semantic activation through Wabou's ordinary JS event
    /// batch. Native controls use this after they have completed their own
    /// pointer or keyboard gesture; no synthetic coordinates are invented.
    pub fn activate(&self, cx: &mut App) {
        self.events().activate(cx);
    }
}

/// One ephemeral element plus optional state retained across Solid frames.
pub struct NativeWidgetMount {
    element: AnyElement,
    entity: Option<AnyEntity>,
    input: Option<NativeWidgetInputHandler>,
}

/// Backend-owned input path for a retained native widget.
pub type NativeWidgetInputHandler = Rc<dyn Fn(UiEvent, &mut Window, &mut App) -> bool + 'static>;

/// Common input contract for retained native widgets.
///
/// Implementations receive the same semantic input used by Wabou's test
/// driver and native event bridge. Returning `true` schedules a GPUI repaint;
/// callers therefore cannot accidentally update retained state without making
/// the result visible. Output events still travel through
/// [`NativeWidgetContext::events`], keeping input and output on the ordinary
/// batched Wabou event path.
pub trait NativeWidgetInput: Sized + 'static {
    /// Apply one semantic input event to retained widget state.
    fn handle_native_input(
        &mut self,
        event: &UiEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool;
}

impl NativeWidgetMount {
    /// Construct a stateless native widget mount.
    #[must_use]
    pub fn stateless(element: AnyElement) -> Self {
        Self {
            element,
            entity: None,
            input: None,
        }
    }

    /// Construct a mount backed by a stable GPUI entity.
    #[must_use]
    pub fn entity<T: 'static>(entity: Entity<T>, element: AnyElement) -> Self {
        Self {
            element,
            entity: Some(entity.into_any()),
            input: None,
        }
    }

    /// Construct a retained widget that can consume semantic test/native
    /// input through the same backend event path as its GPUI element.
    #[must_use]
    pub fn entity_with_input<T: 'static>(
        entity: Entity<T>,
        element: AnyElement,
        input: NativeWidgetInputHandler,
    ) -> Self {
        Self {
            element,
            entity: Some(entity.into_any()),
            input: Some(input),
        }
    }

    /// Construct an interactive retained widget from its input contract.
    ///
    /// This is the preferred constructor for editors, terminals and other
    /// stateful controls. It guarantees that test input and native input use
    /// the same handler and that handled state changes notify GPUI exactly
    /// once.
    #[must_use]
    pub fn interactive_entity<T: NativeWidgetInput>(
        entity: Entity<T>,
        element: AnyElement,
    ) -> Self {
        let input_entity = entity.clone();
        let input = Rc::new(move |event: UiEvent, window: &mut Window, cx: &mut App| {
            let mut handled = false;
            input_entity.update(cx, |state, entity_cx| {
                handled = state.handle_native_input(&event, window, entity_cx);
                if handled {
                    entity_cx.notify();
                }
            });
            handled
        });
        Self::entity_with_input(entity, element, input)
    }

    #[doc(hidden)]
    pub fn into_parts(
        self,
    ) -> (
        AnyElement,
        Option<AnyEntity>,
        Option<NativeWidgetInputHandler>,
    ) {
        (self.element, self.entity, self.input)
    }
}

/// Factory that materializes an application-owned GPUI element for one node.
///
/// GPUI elements are intentionally ephemeral; stable state belongs in GPUI
/// entities or application caches keyed by [`NativeWidgetContext::key`]. The
/// native application context is supplied so factories can create those
/// entities instead of being limited to stateless elements.
pub type NativeWidgetFactory = Arc<
    dyn for<'a> Fn(NativeWidgetContext<'a>, &mut Window, &mut App) -> NativeWidgetMount
        + Send
        + Sync
        + 'static,
>;

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext as _, IntoElement as _, Render, TestAppContext};

    struct Harness;

    impl Render for Harness {
        fn render(
            &mut self,
            _window: &mut Window,
            _cx: &mut Context<Self>,
        ) -> impl gpui::IntoElement {
            gpui::div()
        }
    }

    #[test]
    fn context_exposes_exact_key_and_deterministic_authored_attributes() {
        let key = NodeKey::new(17, 9);
        let attributes = BTreeMap::from([
            (SharedString::from("center-x"), SharedString::from("-0.745")),
            (SharedString::from("iterations"), SharedString::from("96")),
        ]);

        let input = std::rc::Rc::new(|_, _: &mut App| {});
        let context =
            NativeWidgetContext::new(key, &attributes, Some(r#"{"iterations":96}"#), None, input);

        assert_eq!(context.key(), key);
        assert_eq!(context.attribute("center-x"), Some("-0.745"));
        assert_eq!(context.attribute("missing"), None);
        assert_eq!(context.config_json(), Some(r#"{"iterations":96}"#));
        assert_eq!(
            context.attributes().collect::<Vec<_>>(),
            [("center-x", "-0.745"), ("iterations", "96")]
        );
    }

    #[test]
    fn context_distinguishes_initial_unchanged_and_updated_attributes() {
        let key = NodeKey::new(18, 9);
        let original =
            BTreeMap::from([(SharedString::from("cwd"), SharedString::from("/workspace"))]);
        let updated = BTreeMap::from([(SharedString::from("cwd"), SharedString::from("/other"))]);
        let input = std::rc::Rc::new(|_, _: &mut App| {});

        assert!(
            NativeWidgetContext::new(key, &original, None, None, input.clone())
                .attributes_changed()
        );
        assert!(
            !NativeWidgetContext::new_with_previous_attributes(
                key,
                &original,
                Some(&original),
                None,
                None,
                input.clone(),
            )
            .attributes_changed()
        );
        assert!(
            NativeWidgetContext::new_with_previous_attributes(
                key,
                &updated,
                Some(&original),
                None,
                None,
                input,
            )
            .attributes_changed()
        );
    }

    #[gpui::test]
    fn event_sink_preserves_typed_text_and_focus_events(cx: &mut TestAppContext) {
        let received = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let capture = received.clone();
        let input = std::rc::Rc::new(move |event, _: &mut App| {
            capture.borrow_mut().push(event);
        });
        let sink = NativeWidgetEventSink::new(NodeKey::new(8, 2), input);

        cx.update(|app| {
            sink.input_text("你好 GPUI", app);
            sink.focus(true, app);
            sink.focus(false, app);
            sink.text_selection(2, 5, app);
            sink.submit(false, true, app);
        });

        assert_eq!(
            received.borrow().as_slice(),
            [
                crate::ProjectedInputEvent::TextChange {
                    target: NodeKey::new(8, 2),
                    value: "你好 GPUI".into(),
                },
                crate::ProjectedInputEvent::FocusChange {
                    target: NodeKey::new(8, 2),
                    focused: true,
                },
                crate::ProjectedInputEvent::FocusChange {
                    target: NodeKey::new(8, 2),
                    focused: false,
                },
                crate::ProjectedInputEvent::TextSelectionChange {
                    target: NodeKey::new(8, 2),
                    anchor: 2,
                    head: 5,
                },
                crate::ProjectedInputEvent::Submit {
                    target: NodeKey::new(8, 2),
                    secondary: false,
                    shift: true,
                },
            ]
        );
    }

    #[gpui::test]
    fn context_recovers_only_the_retained_entity_type(cx: &mut TestAppContext) {
        struct TerminalState(u32);
        struct OtherState;

        cx.update(|app| {
            let entity = app.new(|_| TerminalState(7));
            let retained = entity.clone().into_any();
            let attributes = BTreeMap::new();
            let input = std::rc::Rc::new(|_, _: &mut App| {});
            let context = NativeWidgetContext::new(
                NodeKey::new(3, 4),
                &attributes,
                None,
                Some(&retained),
                input,
            );

            let recovered = context
                .entity::<TerminalState>()
                .expect("matching entity type");
            assert_eq!(recovered.read(app).0, 7);
            assert!(context.entity::<OtherState>().is_none());
        });
    }

    #[gpui::test]
    fn context_routes_semantic_activation_to_the_exact_node(cx: &mut TestAppContext) {
        let activated = std::rc::Rc::new(std::cell::Cell::new(None));
        let observed = activated.clone();
        cx.update(|app| {
            let input = std::rc::Rc::new(move |event, _: &mut App| {
                if let crate::ProjectedInputEvent::Activate { target } = event {
                    observed.set(Some(target));
                }
            });
            let attributes = BTreeMap::new();
            let key = NodeKey::new(21, 8);
            let context = NativeWidgetContext::new(key, &attributes, None, None, input);
            let events = context.events();
            drop(context);
            events.activate(app);
            assert_eq!(activated.get(), Some(key));
        });
    }

    #[gpui::test]
    fn context_routes_numeric_changes_without_json(cx: &mut TestAppContext) {
        let changed = std::rc::Rc::new(std::cell::Cell::new(None));
        let observed = changed.clone();
        cx.update(|app| {
            let input = std::rc::Rc::new(move |event, _: &mut App| {
                if let crate::ProjectedInputEvent::ValueChange { target, value } = event {
                    observed.set(Some((target, value)));
                }
            });
            let attributes = BTreeMap::new();
            let key = NodeKey::new(22, 8);
            NativeWidgetContext::new(key, &attributes, None, None, input)
                .events()
                .change_f64(42.5, app);
            assert_eq!(changed.get(), Some((key, 42.5)));
        });
    }

    #[gpui::test]
    fn interactive_entity_routes_semantic_input_through_retained_state(cx: &mut TestAppContext) {
        struct EditorState {
            text: String,
        }

        impl NativeWidgetInput for EditorState {
            fn handle_native_input(
                &mut self,
                event: &UiEvent,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> bool {
                let UiEvent::TextInput(text) = event else {
                    return false;
                };
                self.text.push_str(text);
                true
            }
        }

        let (_view, window) = cx.add_window_view(|window, cx| {
            let editor = cx.new(|_| EditorState {
                text: String::new(),
            });
            let mount = NativeWidgetMount::interactive_entity(
                editor.clone(),
                gpui::div().into_any_element(),
            );
            let (_, retained, input) = mount.into_parts();
            assert_eq!(
                retained
                    .expect("interactive entity remains retained")
                    .entity_id(),
                editor.entity_id()
            );
            assert!(input.expect("interactive entity exposes input")(
                UiEvent::TextInput("你好".into()),
                window,
                cx,
            ));
            assert_eq!(editor.read(cx).text, "你好");
            Harness
        });
        window.update(|_, _| {});
    }
}
