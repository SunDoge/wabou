//! Shared formatting for common prefixed scalar text.
use crate::{Binding, BoundState, Computed, Key, State};
use std::rc::Rc;

macro_rules! prefixed_text {
    ($($value:ty),* $(,)?) => {$(
        impl State<$value> {
            /// Display this source after an owned prefix, without reading it now.
            pub fn text_with_prefix(self, prefix: impl Into<String>) -> Binding {
                self.prefixed_text(prefix.into())
            }
            fn prefixed_text(self, prefix: String) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.prefixed.state").scoped(self.key()),
                    compute: Rc::new(move |cx| Ok(format!("{prefix}{}", self.get(cx)?))),
                }
            }
        }
        impl BoundState<'_, '_, $value> {
            /// Own the prefix and source descriptor, never this turn's context.
            pub fn text_with_prefix(&self, prefix: impl Into<String>) -> Binding {
                self.state.text_with_prefix(prefix)
            }
        }
        impl Computed<$value> {
            /// Display this memo after an owned prefix; equal outputs skip formatting.
            pub fn text_with_prefix(self, prefix: impl Into<String>) -> Binding {
                self.prefixed_text(prefix.into())
            }
            fn prefixed_text(self, prefix: String) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.prefixed.computed").scoped(self.key),
                    compute: Rc::new(move |cx| Ok(format!("{prefix}{}", self.get(cx)?))),
                }
            }
        }
    )*};
}
prefixed_text!(i64, f64, bool, String);
