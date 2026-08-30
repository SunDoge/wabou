//! Application-defined GPUI elements mounted behind explicit Wabou tags.

use std::{collections::BTreeMap, sync::Arc};

use gpui::{AnyElement, AnyEntity, App, Entity, SharedString, Window};

use crate::NodeKey;

/// Immutable authored state supplied when materializing a native GPUI widget.
///
/// The node key is generational and remains stable across ordinary Solid
/// updates. Stateful widgets may use it as their GPUI element identity or as a
/// key into an application-owned entity registry.
pub struct NativeWidgetContext<'a> {
    key: NodeKey,
    attributes: &'a BTreeMap<SharedString, SharedString>,
    entity: Option<&'a AnyEntity>,
}

impl<'a> NativeWidgetContext<'a> {
    #[doc(hidden)]
    pub fn new(
        key: NodeKey,
        attributes: &'a BTreeMap<SharedString, SharedString>,
        entity: Option<&'a AnyEntity>,
    ) -> Self {
        Self {
            key,
            attributes,
            entity,
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

    /// Recover the stable GPUI entity retained for this node, when its type
    /// matches the factory's state type.
    #[must_use]
    pub fn entity<T: 'static>(&self) -> Option<Entity<T>> {
        self.entity?.clone().downcast().ok()
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

        let context = NativeWidgetContext::new(key, &attributes, None);

        assert_eq!(context.key(), key);
        assert_eq!(context.attribute("center-x"), Some("-0.745"));
        assert_eq!(context.attribute("missing"), None);
        assert_eq!(
            context.attributes().collect::<Vec<_>>(),
            [("center-x", "-0.745"), ("iterations", "96")]
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
            let context =
                NativeWidgetContext::new(NodeKey::new(3, 4), &attributes, Some(&retained));

            let recovered = context
                .entity::<TerminalState>()
                .expect("matching entity type");
            assert_eq!(recovered.read(app).0, 7);
            assert!(context.entity::<OtherState>().is_none());
        });
    }
}
