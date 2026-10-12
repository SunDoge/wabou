//! Shared, concrete adapters for single-source reactive styles.
//!
//! Owned format parameters can outlive a bound handle without retaining its context:
//! ```
//! use blockui::{Cx, Result, State, ViewStyle};
//! fn owned_style(cx: &Cx<'_>) -> Result<impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static + use<>> {
//!     let source = State::<i64>::new("size", 0).bind(cx)?;
//!     let caption = String::from("Preview");
//!     Ok(source.try_map_style_with(move |value| Ok(ViewStyle {
//!         font_size: 14.0 + value as f32, width: caption.len() as f32 * 20.0,
//!         ..Default::default()
//!     })))
//! }
//! ```
//!
//! Borrowed rendering contexts cannot be captured:
//! ```compile_fail
//! use blockui::{Cx, State, ViewStyle};
//! fn invalid(cx: &Cx<'_>) {
//!     let source = State::<i64>::new("size", 0);
//!     let _style = source.map_style_with(move |_| ViewStyle {
//!         font_size: source.get(cx).unwrap() as f32, ..Default::default()
//!     });
//! }
//! ```
use crate::{BoundState, Computed, Cx, Result, State, ViewStyle};

macro_rules! style_map {
    ($($value:ty),* $(,)?) => {$(
        impl State<$value> {
            fn __owned_style(self, map: Box<dyn Fn($value) -> ViewStyle>)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            {
                move |cx: &Cx<'_>| Ok(map(self.get(cx)?))
            }
            fn __owned_fallible_style(self, map: Box<dyn Fn($value) -> Result<ViewStyle>>)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            {
                move |cx: &Cx<'_>| map(self.get(cx)?)
            }

            /// Map one source with owned style parameters, without retaining a turn context.
            /// Captures are snapshots. Mount with `style_reactive` to evaluate lazily.
            /// For an existing mapper, equal memo outputs skip evaluation; rebuilding
            /// the parent supplies a fresh mapper and its captured parameters.
            pub fn map_style_with<F>(self, map: F)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            where F: Fn($value) -> ViewStyle + 'static
            {
                self.__owned_style(Box::new(map))
            }
            /// Fallible style mapping with owned parameters. Errors reject the turn.
            /// The returned callback retains no borrowed rendering context.
            pub fn try_map_style_with<F>(self, map: F)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            where F: Fn($value) -> Result<ViewStyle> + 'static
            {
                self.__owned_fallible_style(Box::new(map))
            }

            /// Map this state to an independently tracked style.
            /// Pass the result to `style_reactive`. The noncapturing mapper runs
            /// with the current value when the style observer evaluates; creating
            /// the adapter does not read state in the parent builder.
            pub fn map_style(self, map: fn($value) -> ViewStyle)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            {
                move |cx: &Cx<'_>| Ok(map(self.get(cx)?))
            }
            /// Fallible single-source style mapping. Returning an error rejects
            /// the turn without publishing its state or presentation changes.
            pub fn try_map_style(self, map: fn($value) -> Result<ViewStyle>)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            {
                move |cx: &Cx<'_>| map(self.get(cx)?)
            }
        }
        impl BoundState<'_, '_, $value> {
            /// Map one source with owned style parameters, without retaining a turn context.
            /// Captures are snapshots. Mount with `style_reactive` to evaluate lazily.
            /// For an existing mapper, equal memo outputs skip evaluation; rebuilding
            /// the parent supplies a fresh mapper and its captured parameters.
            pub fn map_style_with<F>(&self, map: F)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static + use<F>
            where F: Fn($value) -> ViewStyle + 'static
            {
                self.state.map_style_with(map)
            }
            /// Fallible style mapping with owned parameters. Errors reject the turn.
            /// The returned callback retains no borrowed rendering context.
            pub fn try_map_style_with<F>(&self, map: F)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static + use<F>
            where F: Fn($value) -> Result<ViewStyle> + 'static
            {
                self.state.try_map_style_with(map)
            }

            /// Map without retaining this turn's context. Pass to `style_reactive`.
            pub fn map_style(&self, map: fn($value) -> ViewStyle)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static + use<>
            {
                self.state.map_style(map)
            }
            /// Fallible mapping without retaining this turn's context.
            pub fn try_map_style(&self, map: fn($value) -> Result<ViewStyle>)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static + use<>
            {
                self.state.try_map_style(map)
            }
        }
        impl Computed<$value> {
            fn __owned_style(self, map: Box<dyn Fn($value) -> ViewStyle>)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            {
                move |cx: &Cx<'_>| Ok(map(self.get(cx)?))
            }
            fn __owned_fallible_style(self, map: Box<dyn Fn($value) -> Result<ViewStyle>>)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            {
                move |cx: &Cx<'_>| map(self.get(cx)?)
            }

            /// Map one source with owned style parameters, without retaining a turn context.
            /// Captures are snapshots. Mount with `style_reactive` to evaluate lazily.
            /// For an existing mapper, equal memo outputs skip evaluation; rebuilding
            /// the parent supplies a fresh mapper and its captured parameters.
            pub fn map_style_with<F>(self, map: F)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            where F: Fn($value) -> ViewStyle + 'static
            {
                self.__owned_style(Box::new(map))
            }
            /// Fallible style mapping with owned parameters. Errors reject the turn.
            /// The returned callback retains no borrowed rendering context.
            pub fn try_map_style_with<F>(self, map: F)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            where F: Fn($value) -> Result<ViewStyle> + 'static
            {
                self.__owned_fallible_style(Box::new(map))
            }

            /// Map this memo to an independently tracked style.
            /// Pass the result to `style_reactive`. Unchanged memo outputs skip
            /// the mapper; constructing the adapter does not evaluate the memo.
            pub fn map_style(self, map: fn($value) -> ViewStyle)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            {
                move |cx: &Cx<'_>| Ok(map(self.get(cx)?))
            }
            /// Fallible single-source style mapping. Returning an error rejects
            /// the turn without publishing its state or presentation changes.
            pub fn try_map_style(self, map: fn($value) -> Result<ViewStyle>)
                -> impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static
            {
                move |cx: &Cx<'_>| map(self.get(cx)?)
            }
        }
    )*};
}
style_map!(i64, f64, bool, String, Vec<i64>, Vec<String>);
