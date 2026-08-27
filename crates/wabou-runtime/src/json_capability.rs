//! JSON transport used by application-defined host capabilities.

use std::fmt::Display;
use std::future::Future;

use serde::Serialize;
use serde::de::DeserializeOwned;
use wabou_bindgen::{JsonCapabilityErrorCode, JsonMethod};

/// A namespaced host capability that exposes structured asynchronous methods.
///
/// Generated Wabou clients JSON-encode their single request value. This adapter
/// owns the matching decode, Promise installation, and result envelope so
/// applications do not duplicate transport glue around ordinary Rust async
/// functions.
pub struct JsonCapability<'js> {
    pub(crate) ctx: rquickjs::Ctx<'js>,
    pub(crate) object: rquickjs::Object<'js>,
}

impl<'js> JsonCapability<'js> {
    /// Install a method whose function body can be replaced by Subsecond while
    /// the host process and capability state remain alive.
    ///
    /// This intentionally accepts a function pointer instead of a closure.
    /// Persistent state must be passed in the request or retained by the stable
    /// host rather than captured inside the hot-patched code.
    pub fn hot_method<Request, Response, Error, HandlerFuture>(
        &self,
        method: JsonMethod<Request, Response>,
        handler: fn(Request) -> HandlerFuture,
    ) -> rquickjs::Result<()>
    where
        Request: DeserializeOwned + 'static,
        Response: Serialize + 'static,
        Error: Display + 'static,
        HandlerFuture: Future<Output = Result<Response, Error>> + 'static,
    {
        #[cfg(not(feature = "rust-hot-reload"))]
        return self.method(method, handler);

        #[cfg(feature = "rust-hot-reload")]
        {
            let handler = std::sync::Arc::new(std::sync::Mutex::new(
                dioxus_devtools::subsecond::HotFn::current(handler),
            ));
            self.method(method, move |request| {
                handler
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .call((request,))
            })
        }
    }

    /// Install a hot method with state owned by the stable host process.
    /// Patches replace only `handler`; `state` is cloned into each invocation
    /// and is never reconstructed by the patch loader.
    pub fn hot_method_with<State, Request, Response, Error, HandlerFuture>(
        &self,
        method: JsonMethod<Request, Response>,
        state: State,
        handler: fn(State, Request) -> HandlerFuture,
    ) -> rquickjs::Result<()>
    where
        State: Clone + rquickjs::markers::ParallelSend + 'static,
        Request: DeserializeOwned + 'static,
        Response: Serialize + 'static,
        Error: Display + 'static,
        HandlerFuture: Future<Output = Result<Response, Error>> + 'static,
    {
        #[cfg(not(feature = "rust-hot-reload"))]
        return self.method(method, move |request| handler(state.clone(), request));

        #[cfg(feature = "rust-hot-reload")]
        {
            let handler = std::sync::Arc::new(std::sync::Mutex::new(
                dioxus_devtools::subsecond::HotFn::current(handler),
            ));
            self.method(method, move |request| {
                handler
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .call((state.clone(), request))
            })
        }
    }

    /// Install one async method accepting a JSON-encoded request and returning
    /// Wabou's `{ ok, value | error }` envelope.
    pub fn method<Request, Response, Error, Handler, HandlerFuture>(
        &self,
        method: JsonMethod<Request, Response>,
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
                &format!(
                    "invalid JSON capability method identifier `{}`",
                    method.name()
                ),
            ));
        }
        if self.object.contains_key(method.name())? {
            return Err(rquickjs::Exception::throw_type(
                &self.ctx,
                &format!("duplicate JSON capability method `{}`", method.name()),
            ));
        }
        if method.has_request() {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move |raw: String| {
                    let handler = handler.clone();
                    async move { invoke_json_method(&raw, handler).await }
                }),
            )?;
            self.object.set(method.name(), function)
        } else {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move || {
                    let handler = handler.clone();
                    async move { invoke_json_method("null", handler).await }
                }),
            )?;
            self.object.set(method.name(), function)
        }
    }
}

pub(crate) async fn invoke_json_method<Request, Response, Error, Handler, HandlerFuture>(
    raw: &str,
    handler: Handler,
) -> String
where
    Request: DeserializeOwned,
    Response: Serialize,
    Error: Display,
    Handler: Fn(Request) -> HandlerFuture,
    HandlerFuture: Future<Output = Result<Response, Error>>,
{
    let mut deserializer = serde_json::Deserializer::from_str(raw);
    let mut ignored = Vec::new();
    let request = match serde_ignored::deserialize(&mut deserializer, |path| {
        ignored.push(path.to_string());
    }) {
        Ok(request) => request,
        Err(error) => {
            return json_capability_error(
                JsonCapabilityErrorCode::InvalidRequest,
                format!("invalid capability request: {error}"),
            );
        }
    };
    if let Err(error) = deserializer.end() {
        return json_capability_error(
            JsonCapabilityErrorCode::InvalidRequest,
            format!("invalid capability request: {error}"),
        );
    }
    if !ignored.is_empty() {
        return json_capability_error(
            JsonCapabilityErrorCode::InvalidRequest,
            format!(
                "unknown capability request {}: {}",
                if ignored.len() == 1 {
                    "field"
                } else {
                    "fields"
                },
                ignored.join(", ")
            ),
        );
    }
    match handler(request).await {
        Ok(value) => match serde_json::to_value(value) {
            Ok(value) => serde_json::json!({ "ok": true, "value": value }).to_string(),
            Err(error) => json_capability_error(
                JsonCapabilityErrorCode::ResponseEncodingFailure,
                format!("cannot encode capability response: {error}"),
            ),
        },
        Err(error) => {
            json_capability_error(JsonCapabilityErrorCode::HandlerFailure, error.to_string())
        }
    }
}

fn json_capability_error(code: JsonCapabilityErrorCode, message: String) -> String {
    serde_json::json!({
        "ok": false,
        "error": {
            "code": code.as_str(),
            "message": message,
        },
    })
    .to_string()
}

#[cfg(all(test, feature = "rust-hot-reload"))]
mod hot_reload_tests {
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use wabou_bindgen::JsonMethod;

    use super::JsonCapability;
    use crate::jsrt::JsRuntime;

    #[derive(Deserialize)]
    struct DoubleRequest {
        value: u32,
    }

    #[derive(Serialize)]
    struct DoubleResponse {
        value: u32,
    }

    async fn double(request: DoubleRequest) -> Result<DoubleResponse, String> {
        Ok(DoubleResponse {
            value: request.value * 2,
        })
    }

    async fn count(
        calls: Arc<AtomicU32>,
        request: DoubleRequest,
    ) -> Result<DoubleResponse, String> {
        calls.fetch_add(1, Ordering::Relaxed);
        double(request).await
    }

    #[test]
    fn hot_method_mounts_an_explicit_function_pointer() {
        const DOUBLE: JsonMethod<DoubleRequest, DoubleResponse> = JsonMethod::new("double");
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("hotTest", |ctx, object| {
                JsonCapability { ctx, object }.hot_method(DOUBLE, double)
            })
            .expect("mount hot capability");
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    r#"globalThis.hotResult = undefined;
                    __wabou_capabilities.hotTest.double(JSON.stringify({ value: 21 }))
                      .then(value => globalThis.hotResult = JSON.parse(value).value.value);"#,
                )
            })
            .expect("invoke hot capability");
        while runtime.poll_async_runtime() {}
        let result = runtime
            .with(|ctx| ctx.eval::<Option<u32>, _>("globalThis.hotResult"))
            .expect("read hot capability result");
        assert_eq!(result, Some(42));
    }

    #[test]
    fn hot_method_with_keeps_state_outside_the_replaceable_handler() {
        const COUNT: JsonMethod<DoubleRequest, DoubleResponse> = JsonMethod::new("count");
        let calls = Arc::new(AtomicU32::new(0));
        let mounted = calls.clone();
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("hotState", move |ctx, object| {
                JsonCapability { ctx, object }.hot_method_with(COUNT, mounted.clone(), count)
            })
            .expect("mount stateful hot capability");
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    r#"Promise.all([
                      __wabou_capabilities.hotState.count(JSON.stringify({ value: 1 })),
                      __wabou_capabilities.hotState.count(JSON.stringify({ value: 2 }))
                    ]);"#,
                )
            })
            .expect("invoke stateful hot capability");
        while runtime.poll_async_runtime() {}
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }
}
