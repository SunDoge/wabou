//! Resolve opaque computed return aliases in Rust rather than parsing their names.
use crate::{Computed, Cx, Result};

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "unsupported computed return type `{Self}`",
    note = "computed values support i64, f64, bool, String, Vec<i64>, or Vec<String>, optionally wrapped in blockui::Result<T>; type aliases must resolve to one of these types"
)]
pub trait ComputedOutput: Sized {
    type Value;
    fn into_result(self) -> Result<Self::Value>;
    fn from_result(value: Result<Self::Value>) -> Self;
    fn read(computed: Computed<Self::Value>, cx: &Cx<'_>) -> Self;
}

macro_rules! outputs {
    ($($value:ty),* $(,)?) => {$(
        impl ComputedOutput for $value {
            type Value = $value;
            fn into_result(self) -> Result<Self::Value> { Ok(self) }
            fn from_result(value: Result<Self::Value>) -> Self {
                value.expect("infallible computed evaluation failed")
            }
            fn read(computed: Computed<Self::Value>, cx: &Cx<'_>) -> Self {
                Self::from_result(computed.get(cx))
            }
        }
        impl ComputedOutput for Result<$value> {
            type Value = $value;
            fn into_result(self) -> Result<Self::Value> { self }
            fn from_result(value: Result<Self::Value>) -> Self { value }
            fn read(computed: Computed<Self::Value>, cx: &Cx<'_>) -> Self {
                computed.get(cx)
            }
        }
    )*};
}
outputs!(i64, f64, bool, String, Vec<i64>, Vec<String>);
