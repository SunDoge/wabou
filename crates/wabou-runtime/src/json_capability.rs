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
