//! Direct structured-value transport for application-defined host APIs.

use std::fmt::Display;
use std::future::Future;

use rquickjs::{Ctx, FromJs, IntoJs, Object, Value};
use serde::{Serialize, de::DeserializeOwned};
use wabou_bindgen::HostMethod;

struct SerdeValue<T>(T);

impl<'js, T> FromJs<'js> for SerdeValue<T>
where
    T: DeserializeOwned,
{
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
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

impl<'js, T> IntoJs<'js> for SerdeValue<T>
where
    T: Serialize,
{
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        rquickjs_serde::to_value(ctx.clone(), self.0).map_err(|error| {
            rquickjs::Error::new_into_js_message(
                std::any::type_name::<T>(),
                "JavaScript value",
                error.to_string(),
            )
        })
    }
}

/// A namespace of typed Promise-returning functions using direct QuickJS
/// objects instead of intermediate JSON strings.
pub struct NativeCapability<'js> {
    pub(crate) ctx: Ctx<'js>,
    pub(crate) object: Object<'js>,
}

impl<'js> NativeCapability<'js> {
    /// Install a typed asynchronous native method.
    pub fn method<Request, Response, Error, Handler, HandlerFuture>(
        &self,
        method: HostMethod<Request, Response>,
        handler: Handler,
    ) -> rquickjs::Result<()>
    where
        Request: DeserializeOwned + 'static,
        Response: Serialize + 'static,
        Error: Display + 'static,
        Handler: Fn(Request) -> HandlerFuture + Clone + rquickjs::markers::ParallelSend + 'static,
        HandlerFuture: Future<Output = Result<Response, Error>> + 'static,
    {
        if !wabou_bindgen::is_contract_identifier(method.name()) {
            return Err(rquickjs::Exception::throw_type(
                &self.ctx,
                &format!("invalid native method identifier `{}`", method.name()),
            ));
        }
        if self.object.contains_key(method.name())? {
            return Err(rquickjs::Exception::throw_type(
                &self.ctx,
                &format!("duplicate native method `{}`", method.name()),
            ));
        }

        if method.has_request() {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move |SerdeValue(request): SerdeValue<Request>| {
                    let handler = handler.clone();
                    async move { invoke(handler, request).await }
                }),
            )?;
            self.object.set(method.name(), function)
        } else {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move || {
                    let handler = handler.clone();
                    async move {
                        let request = Request::deserialize(serde::de::value::UnitDeserializer::<
                            serde::de::value::Error,
                        >::new())
                        .map_err(|error| {
                            rquickjs::Error::new_from_js_message(
                                "empty native request",
                                std::any::type_name::<Request>(),
                                error.to_string(),
                            )
                        })?;
                        invoke(handler, request).await
                    }
                }),
            )?;
            self.object.set(method.name(), function)
        }
    }
}

async fn invoke<Request, Response, Error, Handler, HandlerFuture>(
    handler: Handler,
    request: Request,
) -> rquickjs::Result<SerdeValue<Response>>
where
    Response: Serialize,
    Error: Display,
    Handler: Fn(Request) -> HandlerFuture,
    HandlerFuture: Future<Output = Result<Response, Error>>,
{
    handler(request).await.map(SerdeValue).map_err(|error| {
        rquickjs::Error::new_into_js_message(
            "Rust native capability",
            "JavaScript Promise",
            error.to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use wabou_bindgen::HostMethod;

    use super::NativeCapability;
    use crate::jsrt::JsRuntime;

    #[derive(Deserialize)]
    struct DoubleRequest {
        value: u32,
    }

    #[derive(Serialize)]
    struct DoubleResponse {
        value: u32,
    }

    #[test]
    fn structured_values_cross_quickjs_without_json_text() {
        const DOUBLE: HostMethod<DoubleRequest, DoubleResponse> = HostMethod::new("double");
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("nativeTest", |ctx, object| {
                NativeCapability { ctx, object }.method(DOUBLE, |request| async move {
                    Ok::<_, String>(DoubleResponse {
                        value: request.value * 2,
                    })
                })
            })
            .expect("mount native capability");
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    "globalThis.nativeResult = undefined; __wabou_capabilities.nativeTest.double({ value: 21 }).then(value => globalThis.nativeResult = value.value);",
                )
            })
            .expect("invoke native method");
        while runtime.poll_async_runtime() {}
        let result = runtime
            .with(|ctx| ctx.eval::<Option<u32>, _>("globalThis.nativeResult"))
            .expect("read native result");
        assert_eq!(result, Some(42));
    }
}
