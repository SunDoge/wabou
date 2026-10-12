//! Shared formatting for common suffixed scalar text.
use crate::{Binding, BoundState, Computed, Key, State};
use std::rc::Rc;

macro_rules! suffixed_text {
    ($($value:ty),* $(,)?) => {$(
        impl State<$value> {
            /// Display this source before an owned suffix, without reading it now.
            pub fn text_with_suffix(self, suffix: impl Into<String>) -> Binding {
                self.suffixed_text(suffix.into())
            }
            fn suffixed_text(self, suffix: String) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.suffixed.state").scoped(self.key()),
                    compute: Rc::new(move |cx| Ok(format!("{}{suffix}", self.get(cx)?))),
                }
            }
        }
        impl BoundState<'_, '_, $value> {
            /// Own the suffix and source descriptor, never this turn's context.
            pub fn text_with_suffix(&self, suffix: impl Into<String>) -> Binding {
                self.state.text_with_suffix(suffix)
            }
        }
        impl Computed<$value> {
            /// Display this memo before an owned suffix; equal outputs skip formatting.
            pub fn text_with_suffix(self, suffix: impl Into<String>) -> Binding {
                self.suffixed_text(suffix.into())
            }
            fn suffixed_text(self, suffix: String) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.suffixed.computed").scoped(self.key),
                    compute: Rc::new(move |cx| Ok(format!("{}{suffix}", self.get(cx)?))),
                }
            }
        }
    )*};
}
suffixed_text!(i64, f64, bool, String);
