//! Application-defined GPUI elements mounted behind explicit Wabou tags.

use std::{collections::BTreeMap, sync::Arc};

use gpui::{AnyElement, AnyEntity, App, Entity, SharedString, Window};

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
}

/// Immutable authored state supplied when materializing a native GPUI widget.
///
/// The node key is generational and remains stable across ordinary Solid
/// updates. Stateful widgets may use it as their GPUI element identity or as a
/// key into an application-owned entity registry.
pub struct NativeWidgetContext<'a> {
    key: NodeKey,
    attributes: &'a BTreeMap<SharedString, SharedString>,
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
}

impl NativeWidgetMount {
    /// Construct a stateless native widget mount.
    #[must_use]
    pub fn stateless(element: AnyElement) -> Self {
        Self {
            element,
            entity: None,
        }
    }

    /// Construct a mount backed by a stable GPUI entity.
    #[must_use]
    pub fn entity<T: 'static>(entity: Entity<T>, element: AnyElement) -> Self {
        Self {
            element,
            entity: Some(entity.into_any()),
        }
    }

    #[doc(hidden)]
    pub fn into_parts(self) -> (AnyElement, Option<AnyEntity>) {
        (self.element, self.entity)
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
    use gpui::{AppContext as _, TestAppContext};

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
}
