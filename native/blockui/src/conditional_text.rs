//! Shared text selection for boolean reactive sources.
use crate::{Binding, BoundState, Computed, Key, State};
use std::rc::Rc;
impl State<bool> {
    /// Select owned text from this source when the text observer runs.
    pub fn text_when(self, when_true: impl Into<String>, when_false: impl Into<String>) -> Binding {
        self.conditional_text(when_true.into(), when_false.into())
    }
    fn conditional_text(self, when_true: String, when_false: String) -> Binding {
        Binding::NamedCaptured {
            identity: Key::named("identity.binding.conditional.state").scoped(self.key()),
            compute: Rc::new(move |cx| {
                Ok(if self.get(cx)? {
                    when_true.clone()
                } else {
                    when_false.clone()
                })
            }),
        }
    }
}
impl BoundState<'_, '_, bool> {
    /// Own both labels and the source descriptor, never this turn's context.
    pub fn text_when(
        &self,
        when_true: impl Into<String>,
        when_false: impl Into<String>,
    ) -> Binding {
        self.state.text_when(when_true, when_false)
    }
}
impl Computed<bool> {
    /// Select owned text from this memo; equal outputs skip the text observer.
    pub fn text_when(self, when_true: impl Into<String>, when_false: impl Into<String>) -> Binding {
        self.conditional_text(when_true.into(), when_false.into())
    }
    fn conditional_text(self, when_true: String, when_false: String) -> Binding {
        Binding::NamedCaptured {
            identity: Key::named("identity.binding.conditional.computed").scoped(self.key()),
            compute: Rc::new(move |cx| {
                Ok(if self.get(cx)? {
                    when_true.clone()
                } else {
                    when_false.clone()
                })
            }),
        }
    }
}
