//! Excluded prototype for direct typed QuickJS capabilities.
//!
//! The production runtime intentionally does not depend on this crate.

use std::fmt::Display;
use std::future::Future;
use std::marker::PhantomData;

use rquickjs::{Ctx, FromJs, IntoJs, Object, Value};
use serde::{Serialize, de::DeserializeOwned};

/// Zero-sized typed description shared by registration and future codegen.
pub struct NativeMethod<Request, Response> {
    name: &'static str,
    has_request: bool,
    marker: PhantomData<fn(Request) -> Response>,
}

impl<Request, Response> Copy for NativeMethod<Request, Response> {}

impl<Request, Response> Clone for NativeMethod<Request, Response> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Request, Response> NativeMethod<Request, Response> {
    /// Declare a method taking one structured JavaScript value.
    pub const fn new(name: &'static str) -> Self
    where
        Request: DeserializeOwned,
        Response: Serialize,
    {
        Self {
            name,
            has_request: true,
            marker: PhantomData,
        }
    }

    /// Return the JavaScript property name.
    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl<Response> NativeMethod<(), Response> {
    /// Declare a method whose JavaScript function takes no argument.
    pub const fn no_request(name: &'static str) -> Self
    where
        Response: Serialize,
    {
        Self {
            name,
            has_request: false,
            marker: PhantomData,
        }
    }
}

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

/// A namespace object populated with direct-value asynchronous methods.
pub struct NativeCapability<'js> {
    ctx: Ctx<'js>,
    object: Object<'js>,
}

impl<'js> NativeCapability<'js> {
    /// Wrap an object created by the embedding host.
    pub fn new(ctx: Ctx<'js>, object: Object<'js>) -> Self {
        Self { ctx, object }
    }

    /// Install one typed handler as a Promise-returning JavaScript function.
    pub fn method<Request, Response, Error, Handler, HandlerFuture>(
        &self,
        method: NativeMethod<Request, Response>,
        handler: Handler,
    ) -> rquickjs::Result<()>
    where
        Request: DeserializeOwned + 'static,
        Response: Serialize + 'static,
        Error: Display + 'static,
        Handler: Fn(Request) -> HandlerFuture + Clone + rquickjs::markers::ParallelSend + 'static,
        HandlerFuture: Future<Output = Result<Response, Error>> + 'static,
    {
        validate_identifier(&self.ctx, method.name)?;
        if self.object.contains_key(method.name)? {
            return Err(rquickjs::Exception::throw_type(
                &self.ctx,
                &format!("duplicate native method `{}`", method.name),
            ));
        }

        if method.has_request {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move |SerdeValue(request): SerdeValue<Request>| {
                    let handler = handler.clone();
                    async move { invoke(handler, request).await }
                }),
            )?;
            self.object.set(method.name, function)
        } else {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move || {
                    let handler = handler.clone();
                    async move {
                        let request = Request::deserialize(
                            serde::de::value::UnitDeserializer::<serde::de::value::Error>::new(),
                        )
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
            self.object.set(method.name, function)
        }
    }
}

async fn invoke<Request, Response, Error, Handler, HandlerFuture>(
    handler: Handler,
    request: Request,
) -> rquickjs::Result<SerdeValue<Response>>
where
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

fn validate_identifier(ctx: &Ctx<'_>, name: &str) -> rquickjs::Result<()> {
    let mut characters = name.chars();
    let valid = name != "__proto__"
        && characters
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric());
    if valid {
        Ok(())
    } else {
        Err(rquickjs::Exception::throw_type(
            ctx,
            &format!("invalid native method identifier `{name}`"),
        ))
    }
}
