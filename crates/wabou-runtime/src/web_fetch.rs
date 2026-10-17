//! Rust-owned HTTP client backing the small Web Fetch surface in `@wabou/core`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rquickjs::{Ctx, IntoJs, Object};
use tokio_util::sync::CancellationToken;

type JsResult<T> = rquickjs::Result<T>;

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct FetchInit {
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
}

pub(crate) struct FetchResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    url: String,
    redirected: bool,
}

impl<'js> IntoJs<'js> for FetchResponse {
    fn into_js(self, ctx: &Ctx<'js>) -> JsResult<rquickjs::Value<'js>> {
        let value = Object::new(ctx.clone())?;
        value.set("status", self.status)?;
        value.set("statusText", self.status_text)?;
        let headers = Object::new(ctx.clone())?;
        for (name, value) in self.headers {
            headers.set(name, value)?;
        }
        value.set("headers", headers)?;
        value.set("body", crate::host_ffi::OwnedJsBytes(self.body))?;
        value.set("url", self.url)?;
        value.set("redirected", self.redirected)?;
        Ok(value.into_value())
    }
}

#[derive(Clone)]
pub(crate) struct FetchService {
    client: reqwest::Client,
    active: Arc<Mutex<HashMap<u32, CancellationToken>>>,
}

impl FetchService {
    pub(crate) fn new() -> JsResult<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|error| host_error("fetch client", "HTTP client", error.to_string()))?;
        Ok(Self {
            client,
            active: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub(crate) fn start(&self, id: u32) {
        self.active
            .lock()
            .expect("fetch cancellation registry poisoned")
            .insert(id, CancellationToken::new());
    }

    pub(crate) fn cancel(&self, id: u32) {
        if let Some(token) = self
            .active
            .lock()
            .expect("fetch cancellation registry poisoned")
            .get(&id)
        {
            token.cancel();
        }
    }

    pub(crate) async fn request(
        self,
        url: String,
        init_json: String,
        id: Option<u32>,
        body: Option<Vec<u8>>,
    ) -> JsResult<FetchResponse> {
        let token = id.and_then(|id| {
            self.active
                .lock()
                .expect("fetch cancellation registry poisoned")
                .get(&id)
                .cloned()
        });
        let result = if let Some(token) = token {
            tokio::select! {
                biased;
                _ = token.cancelled() => Err(host_error("fetch", "response", "request aborted")),
                result = send_request(self.client.clone(), url, init_json, body) => result,
            }
        } else {
            send_request(self.client.clone(), url, init_json, body).await
        };
        if let Some(id) = id {
            self.active
                .lock()
                .expect("fetch cancellation registry poisoned")
                .remove(&id);
        }
        result
    }
}

async fn send_request(
    client: reqwest::Client,
    url: String,
    init_json: String,
    body: Option<Vec<u8>>,
) -> JsResult<FetchResponse> {
    let init: FetchInit = serde_json::from_str(&init_json)
        .map_err(|error| host_error("fetch init", "valid JSON", error.to_string()))?;
    let method = init
        .method
        .as_deref()
        .unwrap_or("GET")
        .parse::<reqwest::Method>()
        .map_err(|error| host_error("fetch method", "HTTP method", error.to_string()))?;
    let original_url = reqwest::Url::parse(&url)
        .map_err(|error| host_error("fetch URL", "HTTP URL", error.to_string()))?;
    if !matches!(original_url.scheme(), "http" | "https") {
        return Err(host_error(
            "fetch URL",
            "http or https URL",
            format!("unsupported scheme `{}`", original_url.scheme()),
        ));
    }
    let mut request = client.request(method, original_url.clone());
    if let Some(headers) = init.headers {
        for (name, value) in headers {
            request = request.header(name, value);
        }
    }
    if let Some(body) = body {
        request = request.body(body);
    }
    let response = request
        .send()
        .await
        .map_err(|error| host_error("fetch", "response", error.to_string()))?;
    let final_url = response.url().clone();
    let redirected = final_url != original_url;
    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect::<HashMap<_, _>>();
    let body = response
        .bytes()
        .await
        .map_err(|error| host_error("fetch body", "bytes", error.to_string()))?
        .to_vec();
    Ok(FetchResponse {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or("").to_owned(),
        headers,
        body,
        url: final_url.to_string(),
        redirected,
    })
}

fn host_error(from: &'static str, to: &'static str, message: impl Into<String>) -> rquickjs::Error {
    rquickjs::Error::new_from_js_message(from, to, message.into())
}
