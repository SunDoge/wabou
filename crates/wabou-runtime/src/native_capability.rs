//! Direct structured-value transport for application-defined host APIs.

use std::fmt::Display;
use std::future::Future;

use rquickjs::{Ctx, FromJs, IntoJs, Object, Value};
use serde::{Serialize, de::DeserializeOwned};
use wabou_bindgen::{HostMethod, JsonMethod};

use crate::json_capability::JsonCapability;

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
    /// Install a typed synchronous native method.
    ///
    /// Use this for bounded in-memory/bootstrap reads that must be available
    /// during the initial Solid render. Operations that can wait on IO belong
    /// in [`Self::method`] so they do not block the UI thread.
    pub fn sync_method<Request, Response, Error, Handler>(
        &self,
        method: HostMethod<Request, Response>,
        handler: Handler,
    ) -> rquickjs::Result<()>
    where
        Request: DeserializeOwned + 'static,
        Response: Serialize + 'static,
        Error: Display + 'static,
        Handler: Fn(Request) -> Result<Response, Error>
            + Clone
            + rquickjs::markers::ParallelSend
            + 'static,
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
                move |SerdeValue(request): SerdeValue<Request>| {
                    invoke_sync(handler.clone(), request)
                },
            )?;
            self.object.set(method.name(), function)
        } else {
            let function = rquickjs::Function::new(self.ctx.clone(), move || {
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
                invoke_sync(handler.clone(), request)
            })?;
            self.object.set(method.name(), function)
        }
    }

    /// Install a JSON-coded method in this capability namespace.
    ///
    /// JSON is an optional codec for low-frequency or dynamic DTOs; it does
    /// not require a second capability namespace or registration model.
    pub fn json_method<Request, Response, Error, Handler, HandlerFuture>(
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
        JsonCapability {
            ctx: self.ctx.clone(),
            object: self.object.clone(),
        }
        .method(method, handler)
    }

    /// Install a JSON-coded method whose function body can be hot-patched.
    ///
    /// This is the JSON codec counterpart of [`Self::hot_method`]. The
    /// capability namespace remains the same; only this method's request and
    /// response use JSON text on the QuickJS boundary.
    pub fn json_hot_method<Request, Response, Error, HandlerFuture>(
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
        JsonCapability {
            ctx: self.ctx.clone(),
            object: self.object.clone(),
        }
        .hot_method(method, handler)
    }

    /// Install a hot JSON-coded method whose state remains owned by the host.
    pub fn json_hot_method_with<State, Request, Response, Error, HandlerFuture>(
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
        JsonCapability {
            ctx: self.ctx.clone(),
            object: self.object.clone(),
        }
        .hot_method_with(method, state, handler)
    }

    /// Install a typed method whose function body can be replaced by
    /// Subsecond without restarting the Wabou host.
    ///
    /// Only function pointers are accepted so long-lived state remains owned
    /// by the stable host instead of a replaceable closure environment.
    pub fn hot_method<Request, Response, Error, HandlerFuture>(
        &self,
        method: HostMethod<Request, Response>,
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

    /// Install a hot typed method with state retained by the stable host.
    pub fn hot_method_with<State, Request, Response, Error, HandlerFuture>(
        &self,
        method: HostMethod<Request, Response>,
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

fn invoke_sync<Request, Response, Error, Handler>(
    handler: Handler,
    request: Request,
) -> rquickjs::Result<SerdeValue<Response>>
where
    Response: Serialize,
    Error: Display,
    Handler: Fn(Request) -> Result<Response, Error>,
{
    handler(request).map(SerdeValue).map_err(|error| {
        rquickjs::Error::new_into_js_message(
            "Rust native capability",
            "JavaScript value",
            error.to_string(),
        )
    })
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
    use wabou_bindgen::{HostMethod, JsonMethod};

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

    #[test]
    fn synchronous_native_methods_return_during_the_call() {
        const DOUBLE: HostMethod<DoubleRequest, DoubleResponse> = HostMethod::new("doubleSync");
        const READY: HostMethod<(), DoubleResponse> = HostMethod::no_request("readySync");
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("syncTest", |ctx, object| {
                let capability = NativeCapability { ctx, object };
                capability.sync_method(DOUBLE, |request: DoubleRequest| {
                    Ok::<_, String>(DoubleResponse {
                        value: request.value * 2,
                    })
                })?;
                capability.sync_method(READY, |(): ()| Ok::<_, String>(DoubleResponse { value: 7 }))
            })
            .expect("mount synchronous native capability");

        let result = runtime
            .with(|ctx| {
                ctx.eval::<String, _>(
                    "JSON.stringify([__wabou_capabilities.syncTest.doubleSync({ value: 21 }).value, __wabou_capabilities.syncTest.readySync().value])",
                )
            })
            .expect("invoke synchronous native methods");
        assert_eq!(result, "[42,7]");
        assert!(
            !runtime.poll_async_runtime(),
            "sync calls must not enqueue jobs"
        );
    }

    #[test]
    fn direct_and_json_methods_share_one_capability_namespace() {
        const DIRECT: HostMethod<DoubleRequest, DoubleResponse> = HostMethod::new("direct");
        const JSON: JsonMethod<DoubleRequest, DoubleResponse> = JsonMethod::new("json");
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("mixedTest", |ctx, object| {
                let capability = NativeCapability { ctx, object };
                capability.method(DIRECT, |request| async move {
                    Ok::<_, String>(DoubleResponse {
                        value: request.value * 2,
                    })
                })?;
                capability.json_method(JSON, |request| async move {
                    Ok::<_, String>(DoubleResponse {
                        value: request.value * 3,
                    })
                })
            })
            .expect("mount mixed capability");
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    r#"
                    globalThis.mixedResult = undefined;
                    Promise.all([
                      __wabou_capabilities.mixedTest.direct({ value: 4 }),
                      __wabou_capabilities.mixedTest.json('{"value":4}').then(JSON.parse),
                    ]).then(([direct, json]) => {
                      globalThis.mixedResult = direct.value + json.value.value;
                    });
                    "#,
                )
            })
            .expect("invoke mixed methods");
        while runtime.poll_async_runtime() {}
        let result = runtime
            .with(|ctx| ctx.eval::<Option<u32>, _>("globalThis.mixedResult"))
            .expect("read mixed result");
        assert_eq!(result, Some(20));
    }

    #[test]
    fn boot_time_native_method_settles_after_the_host_wake_is_installed() {
        const AGENTS: HostMethod<(), Vec<String>> = HostMethod::no_request("agents");
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("bootTest", |ctx, object| {
                NativeCapability { ctx, object }.method(AGENTS, |(): ()| async move {
                    Ok::<_, String>(vec!["agent-1".to_owned()])
                })
            })
            .expect("mount native capability");
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    "globalThis.bootAgents = undefined; __wabou_capabilities.bootTest.agents().then(value => globalThis.bootAgents = value[0]);",
                )
            })
            .expect("invoke boot native method");

        let wake_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let callback_count = wake_count.clone();
        runtime.set_wake_callback(std::sync::Arc::new(move || {
            callback_count.fetch_add(1, std::sync::atomic::Ordering::Release);
        }));
        for _ in 0..8 {
            runtime.take_async_wake();
            if !runtime.poll_async_runtime() {
                break;
            }
        }
        assert_eq!(
            runtime
                .with(|ctx| ctx.eval::<Option<String>, _>("globalThis.bootAgents"))
                .expect("read boot agents")
                .as_deref(),
            Some("agent-1")
        );
        assert!(wake_count.load(std::sync::atomic::Ordering::Acquire) >= 1);
    }
}
