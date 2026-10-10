//! Concrete fallible single-source text adapters.
use crate::{Binding, BoundState, Computed, Key, Result, State};
use std::rc::Rc;

macro_rules! try_text_map {
    ($($value:ty),* $(,)?) => {$(
        impl State<$value> {
            /// Map one source to fallible text, without reading it during construction.
            /// Returning an error rejects the turn and retains committed state/text.
            pub fn try_map_text(self, map: fn($value) -> Result<String>) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.fallible.state").scoped(self.key()),
                    compute: Rc::new(move |cx| map(self.get(cx)?)),
                }
            }
        }
        impl BoundState<'_, '_, $value> {
            /// Fallible text mapping without retaining this turn's context.
            pub fn try_map_text(&self, map: fn($value) -> Result<String>) -> Binding {
                self.state.try_map_text(map)
            }
        }
        impl Computed<$value> {
            /// Map a memo to fallible text; equal outputs skip the mapper.
            pub fn try_map_text(self, map: fn($value) -> Result<String>) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.fallible.computed").scoped(self.key),
                    compute: Rc::new(move |cx| map(self.get(cx)?)),
                }
            }
        }
    )*};
}
try_text_map!(i64, f64, bool, String, Vec<i64>, Vec<String>);
