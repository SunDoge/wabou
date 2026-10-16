//! Serde-backed values crossing the QuickJS boundary.

use rquickjs::{Ctx, FromJs, IntoJs, Value};
use serde::{Serialize, de::DeserializeOwned};

/// A Rust value encoded as an ordinary JavaScript value through Serde.
///
/// Use this at explicit host-function boundaries when deriving `FromJs` or
/// `IntoJs` for every data-transfer object would add noise. It does not use a
/// JSON string envelope.
pub struct JsValue<T>(pub T);

impl<'js, T> FromJs<'js> for JsValue<T>
where
    T: DeserializeOwned,
{
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let _measurement = wabou_telemetry::span("host.transport", "request.decode");
        rquickjs_serde::from_value_strict(value)
            .map(Self)
            .map_err(|error| {
                rquickjs::Error::new_from_js_message(
                    "JavaScript value",
                    std::any::type_name::<T>(),
                    error.to_string(),
                )
            })
    }
}

impl<'js, T> IntoJs<'js> for JsValue<T>
where
    T: Serialize,
{
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let _measurement = wabou_telemetry::span("host.transport", "response.encode");
        rquickjs_serde::to_value(ctx.clone(), self.0).map_err(|error| {
            rquickjs::Error::new_into_js_message(
                std::any::type_name::<T>(),
                "JavaScript value",
                error.to_string(),
            )
        })
    }
}
