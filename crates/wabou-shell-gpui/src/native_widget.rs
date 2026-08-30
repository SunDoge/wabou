//! Application-defined GPUI elements mounted behind explicit Wabou tags.

use std::{collections::BTreeMap, sync::Arc};

use gpui::{AnyElement, App, SharedString, Window};

use crate::NodeKey;

/// Immutable authored state supplied when materializing a native GPUI widget.
///
/// The node key is generational and remains stable across ordinary Solid
/// updates. Stateful widgets may use it as their GPUI element identity or as a
/// key into an application-owned entity registry.
pub struct NativeWidgetContext<'a> {
    key: NodeKey,
    attributes: &'a BTreeMap<SharedString, SharedString>,
}

impl<'a> NativeWidgetContext<'a> {
    #[doc(hidden)]
    pub fn new(key: NodeKey, attributes: &'a BTreeMap<SharedString, SharedString>) -> Self {
        Self { key, attributes }
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
}

/// Factory that materializes an application-owned GPUI element for one node.
///
/// GPUI elements are intentionally ephemeral; stable state belongs in GPUI
/// entities or application caches keyed by [`NativeWidgetContext::key`]. The
/// native application context is supplied so factories can create those
/// entities instead of being limited to stateless elements.
pub type NativeWidgetFactory = Arc<
    dyn for<'a> Fn(NativeWidgetContext<'a>, &mut Window, &mut App) -> AnyElement
        + Send
        + Sync
        + 'static,
>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_exposes_exact_key_and_deterministic_authored_attributes() {
        let key = NodeKey::new(17, 9);
        let attributes = BTreeMap::from([
            (SharedString::from("center-x"), SharedString::from("-0.745")),
            (SharedString::from("iterations"), SharedString::from("96")),
        ]);

        let context = NativeWidgetContext::new(key, &attributes);

        assert_eq!(context.key(), key);
        assert_eq!(context.attribute("center-x"), Some("-0.745"));
        assert_eq!(context.attribute("missing"), None);
        assert_eq!(
            context.attributes().collect::<Vec<_>>(),
            [("center-x", "-0.745"), ("iterations", "96")]
        );
    }
}
