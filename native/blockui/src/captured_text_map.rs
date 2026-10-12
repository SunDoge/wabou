//! Owned format parameters stay within the current implementation.
use crate::{Binding, BoundState, Computed, Key, Result, State};
use std::rc::Rc;

macro_rules! captured_text_map {
    ($($value:ty),* $(,)?) => {$(
        impl State<$value> {
            fn __owned_text(self, map: Box<dyn Fn($value) -> String>) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.fallible.state").scoped(self.key()),
                    compute: Rc::new(move |cx| Ok(map(self.get(cx)?))),
                }
            }
            fn __owned_fallible_text(self, map: Box<dyn Fn($value) -> Result<String>>) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.fallible.state").scoped(self.key()),
                    compute: Rc::new(move |cx| map(self.get(cx)?)),
                }
            }

            /// Map one state source with owned format parameters.
            /// The mapper is lazy and implementation-local. Captures are snapshots;
            /// only reads of this source create reactive dependencies.
            /// Use `map_text` for a noncapturing function or `Binding::captured`
            /// when the computation must read several state sources.
            pub fn map_text_with(self, map: impl Fn($value) -> String + 'static) -> Binding {
                self.__owned_text(Box::new(map))
            }
            /// Fallible captured mapping. Errors reject the actor turn.
            /// Captures must be owned; a borrowed rendering context cannot escape.
            pub fn try_map_text_with(self, map: impl Fn($value) -> Result<String> + 'static) -> Binding {
                self.__owned_fallible_text(Box::new(map))
            }
        }
        impl BoundState<'_, '_, $value> {
            /// Map the source with owned parameters without retaining its context.
            pub fn map_text_with(&self, map: impl Fn($value) -> String + 'static) -> Binding {
                self.state.map_text_with(map)
            }
            /// Fallible mapping with owned parameters, evaluated with a fresh context.
            pub fn try_map_text_with(&self, map: impl Fn($value) -> Result<String> + 'static) -> Binding {
                self.state.try_map_text_with(map)
            }
        }
        impl Computed<$value> {
            fn __owned_text(self, map: Box<dyn Fn($value) -> String>) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.fallible.computed").scoped(self.key),
                    compute: Rc::new(move |cx| Ok(map(self.get(cx)?))),
                }
            }
            fn __owned_fallible_text(self, map: Box<dyn Fn($value) -> Result<String>>) -> Binding {
                Binding::NamedCaptured {
                    identity: Key::named("identity.binding.fallible.computed").scoped(self.key),
                    compute: Rc::new(move |cx| map(self.get(cx)?)),
                }
            }

            /// Map a memo with owned parameters. Equal memo outputs skip an existing
            /// mounted mapper; rebuilding the parent supplies and evaluates a new one.
            /// Captured parameters are snapshots, not additional reactive sources.
            pub fn map_text_with(self, map: impl Fn($value) -> String + 'static) -> Binding {
                self.__owned_text(Box::new(map))
            }
            /// Fallible memo mapping with implementation-local owned parameters.
            pub fn try_map_text_with(self, map: impl Fn($value) -> Result<String> + 'static) -> Binding {
                self.__owned_fallible_text(Box::new(map))
            }
        }
    )*};
}
captured_text_map!(i64, f64, bool, String, Vec<i64>, Vec<String>);
