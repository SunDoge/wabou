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
        let _measurement = wabou_telemetry::span("capability.transport", "request.decode");
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
        let _measurement = wabou_telemetry::span("capability.transport", "response.encode");
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
    /// Construct a capability adapter over a runtime-owned namespace object.
    ///
    /// This is intended for alternate Wabou runtime hosts. Applications should
    /// receive the adapter from `HostBuilder::capability` instead of calling it.
    #[doc(hidden)]
    pub fn from_runtime_parts(ctx: Ctx<'js>, object: Object<'js>) -> Self {
        Self { ctx, object }
    }

    fn trace_name(&self, name: &str) -> std::sync::Arc<str> {
        let namespace: Option<String> = self.object.get("__wabou_namespace").unwrap_or(None);
        namespace.map_or_else(
            || std::sync::Arc::from(name),
            |namespace| std::sync::Arc::from(format!("{namespace}.{name}")),
        )
    }

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

        let trace_name = self.trace_name(method.name());
        if method.has_request() {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                move |SerdeValue(request): SerdeValue<Request>| {
                    invoke_sync(&trace_name, handler.clone(), request)
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
                invoke_sync(&trace_name, handler.clone(), request)
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
        self.install(method.name(), method.has_request(), handler)
    }

    fn install<Request, Response, Error, Handler, HandlerFuture>(
        &self,
        name: &str,
        has_request: bool,
        handler: Handler,
    ) -> rquickjs::Result<()>
    where
        Request: DeserializeOwned + 'static,
        Response: Serialize + 'static,
        Error: Display + 'static,
        Handler: Fn(Request) -> HandlerFuture + Clone + rquickjs::markers::ParallelSend + 'static,
        HandlerFuture: Future<Output = Result<Response, Error>> + 'static,
    {
        if !wabou_bindgen::is_contract_identifier(name) {
            return Err(rquickjs::Exception::throw_type(
                &self.ctx,
                &format!("invalid native method identifier `{}`", name),
            ));
        }
        if self.object.contains_key(name)? {
            return Err(rquickjs::Exception::throw_type(
                &self.ctx,
                &format!("duplicate native method `{}`", name),
            ));
        }

        let trace_name = self.trace_name(name);
        if has_request {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move |SerdeValue(request): SerdeValue<Request>| {
                    let handler = handler.clone();
                    let trace_name = trace_name.clone();
                    async move { invoke(&trace_name, handler, request).await }
                }),
            )?;
            self.object.set(name, function)
        } else {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move || {
                    let handler = handler.clone();
                    let trace_name = trace_name.clone();
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
                        invoke(&trace_name, handler, request).await
                    }
                }),
            )?;
            self.object.set(name, function)
        }
    }
}

fn invoke_sync<Request, Response, Error, Handler>(
    name: &str,
    handler: Handler,
    request: Request,
) -> rquickjs::Result<SerdeValue<Response>>
where
    Response: Serialize,
    Error: Display,
    Handler: Fn(Request) -> Result<Response, Error>,
{
    let mut measurement = wabou_telemetry::span("capability.sync", name);
    let result = handler(request);
    measurement.attribute("failed", u64::from(result.is_err()));
    result.map(SerdeValue).map_err(|error| {
        rquickjs::Error::new_into_js_message(
            "Rust native capability",
            "JavaScript value",
            error.to_string(),
        )
    })
}

async fn invoke<Request, Response, Error, Handler, HandlerFuture>(
    name: &str,
    handler: Handler,
    request: Request,
) -> rquickjs::Result<SerdeValue<Response>>
where
    Response: Serialize,
    Error: Display,
    Handler: Fn(Request) -> HandlerFuture,
    HandlerFuture: Future<Output = Result<Response, Error>>,
{
    let mut measurement = wabou_telemetry::span("capability.async", name);
    let result = handler(request).await;
    measurement.attribute("failed", u64::from(result.is_err()));
    result.map(SerdeValue).map_err(|error| {
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
    use crate::{ResidentResources, ResourceKey, jsrt::JsRuntime};

    #[derive(Deserialize)]
    struct DoubleRequest {
        value: u32,
    }

    #[derive(Serialize)]
    struct DoubleResponse {
        value: u32,
    }

    enum CounterResource {}

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
    fn resident_resource_handles_rebind_without_resetting_native_state() {
        const OPEN: HostMethod<(), ResourceKey<CounterResource>> = HostMethod::no_request("open");
        const INCREMENT: HostMethod<ResourceKey<CounterResource>, u32> =
            HostMethod::new("increment");
        let resources = ResidentResources::<CounterResource, u32>::new();
        let open_resources = resources.clone();
        let increment_resources = resources.clone();
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("residentTest", move |ctx, object| {
                let capability = NativeCapability { ctx, object };
                let resources = open_resources.clone();
                capability.sync_method(OPEN, move |(): ()| {
                    Ok::<_, String>(resources.get_or_insert_with("counter".to_owned(), || 0))
                })?;
                let resources = increment_resources.clone();
                capability.sync_method(INCREMENT, move |key| {
                    resources
                        .with_mut(key, |count| {
                            *count += 1;
                            *count
                        })
                        .ok_or_else(|| "stale counter handle".to_owned())
                })
            })
            .expect("mount resident capability");

        let result = runtime
            .with(|ctx| {
                ctx.eval::<String, _>(
                    r#"
                    const beforeReload = __wabou_capabilities.residentTest.open();
                    const first = __wabou_capabilities.residentTest.increment(beforeReload);
                    // A replacement UI module no longer has the original variable.
                    const afterReload = __wabou_capabilities.residentTest.open();
                    const second = __wabou_capabilities.residentTest.increment(afterReload);
                    JSON.stringify({ beforeReload, afterReload, first, second });
                    "#,
                )
            })
            .expect("use resource before and after a UI replacement");
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(result["beforeReload"], result["afterReload"]);
        assert_eq!(result["first"], 1);
        assert_eq!(result["second"], 2);
        assert_eq!(resources.len(), 1);
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
