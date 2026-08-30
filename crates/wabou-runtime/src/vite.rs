//! Vite dev-server integration: module fetching + HMR.
//!
//! Ported from blitz-js's `vite.rs` (host-agnostic). In dev mode the rquickjs
//! runtime gets a module [`Loader`] + [`Resolver`] that fetch ESM from the
//! Vite dev server; the HMR client connects to Vite's WebSocket, prefetches
//! updated module sources, and forwards them to the [`crate::Applier`] via a
//! [`crate::ReloadHandle`].

use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use rquickjs::loader::{ImportAttributes, Loader, Resolver};
use rquickjs::{Ctx, Error, Function, Module, Result};
use url::Url;

pub(crate) const HMR_CLIENT: &str = include_str!("gen/vite-client.js");

pub(crate) struct ViteResolver {
    origin: Url,
}

impl ViteResolver {
    pub(crate) fn new(origin: Url) -> Self {
        Self { origin }
    }

    fn resolve_url(&self, base: &str, name: &str) -> Option<Url> {
        if let Ok(url) = Url::parse(name) {
            return Some(url);
        }
        if name.starts_with('/') {
            return self.origin.join(name).ok();
        }
        Url::parse(base)
            .ok()
            .and_then(|base| base.join(name).ok())
            .or_else(|| self.origin.join(name).ok())
    }
}

impl Resolver for ViteResolver {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> Result<String> {
        self.resolve_url(base, name)
            .map(Into::into)
            .ok_or_else(|| Error::new_resolving_message(base, name, "invalid Vite module URL"))
    }
}

#[derive(Clone, Default)]
pub(crate) struct ViteModuleCache {
    sources: Arc<Mutex<HashMap<String, String>>>,
}

/// All Vite-related state for one JsRuntime: the dev-server origin, the
/// module source cache (shared with the Loader + HMR prefetch), and the
/// methods that drive boot + hot-module replacement.
pub(crate) struct ViteState {
    origin: Url,
    cache: ViteModuleCache,
}

impl ViteState {
    pub(crate) fn new(origin: Url) -> Self {
        Self {
            cache: ViteModuleCache::default(),
            origin,
        }
    }

    /// Install the Vite module loader + resolver onto the runtime so that
    /// `import` inside JS fetches modules from the dev server.
    pub(crate) fn install_loader(&self, rt: &rquickjs::AsyncRuntime) -> rquickjs::Result<()> {
        futures_lite::future::block_on(rt.set_loader(
            ViteResolver::new(self.origin.clone()),
            ViteLoader::new(self.cache.clone()),
        ));
        Ok(())
    }

    /// Evaluate the Vite entry module (`import "<entry>"`). Called once after
    /// the context is ready to kick off the app's initial render.
    pub(crate) fn boot<'js>(&self, ctx: &rquickjs::Ctx<'js>, entry: &str) -> rquickjs::Result<()> {
        use rquickjs::CatchResultExt;
        let entry_url = self
            .origin
            .join(entry)
            .map_err(|_| rquickjs::Error::Unknown)?
            .to_string();
        let entry_literal =
            serde_json::to_string(&entry_url).map_err(|_| rquickjs::Error::Unknown)?;
        let result = Module::evaluate(
            ctx.clone(),
            "wabou-runtime:vite-entry",
            format!("import {entry_literal};"),
        )
        .and_then(|promise| promise.finish::<()>());
        match result.catch(ctx) {
            Ok(()) => Ok(()),
            Err(caught) => {
                match &caught {
                    rquickjs::CaughtError::Exception(exception) => {
                        tracing::error!(
                            entry = %entry_url,
                            message = exception.message().as_deref().unwrap_or("unknown"),
                            stack = exception.stack().as_deref().unwrap_or("unavailable"),
                            "failed to evaluate Vite entry"
                        );
                    }
                    _ => {
                        tracing::error!(entry = %entry_url, error = %caught, "failed to evaluate Vite entry");
                    }
                }
                Err(caught.throw(ctx))
            }
        }
    }

    /// Cache a hot-updated module's source (keyed by URL + timestamp) and call
    /// the JS `__wabou_apply_hmr` hook to reload it. Returns whether the JS
    /// side accepted the update (true) or needs a full reload (false).
    pub(crate) fn apply_hmr<'js>(
        &self,
        ctx: &rquickjs::Ctx<'js>,
        path: &str,
        accepted_path: &str,
        timestamp: u64,
        source: String,
    ) -> rquickjs::Result<bool> {
        let mut update_url = self
            .origin
            .join(accepted_path)
            .map_err(|_| rquickjs::Error::Unknown)?;
        update_url
            .query_pairs_mut()
            .append_pair("t", &timestamp.to_string());
        self.cache.insert(update_url.into(), source);
        let apply: Function = ctx.globals().get("__wabou_apply_hmr")?;
        let promise: rquickjs::Promise = apply.call((path, accepted_path, timestamp))?;
        promise.finish()
    }

    /// Re-import the app entry after a full reload. Uses a cache-busting query
    /// so the loader does not reuse a stale entry module record.
    pub(crate) fn boot_full_reload<'js>(
        &self,
        ctx: &rquickjs::Ctx<'js>,
        entry: &str,
    ) -> rquickjs::Result<()> {
        use rquickjs::CatchResultExt;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Build a fresh hot graph, but retain a reversible snapshot until the
        // entry has evaluated. A transient Vite transform error must not poison
        // the last-good graph used by the next save.
        if let Ok(begin) = ctx
            .globals()
            .get::<_, Function>("__wabou_hmr_begin_full_reload")
        {
            let _: () = begin.call(())?;
        }

        self.cache.clear();

        let entry_url = self
            .origin
            .join(entry)
            .map_err(|_| rquickjs::Error::Unknown)?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let busted = format!(
            "{}{}t={ts}",
            entry_url,
            if entry_url.query().is_some() {
                "&"
            } else {
                "?"
            },
        );
        let entry_literal = serde_json::to_string(&busted).map_err(|_| rquickjs::Error::Unknown)?;
        let result = Module::evaluate(
            ctx.clone(),
            "wabou-runtime:vite-full-reload",
            format!("import {entry_literal};"),
        )
        .and_then(|promise| promise.finish::<()>());
        match result.catch(ctx) {
            Ok(()) => {
                if let Ok(commit) = ctx
                    .globals()
                    .get::<_, Function>("__wabou_hmr_commit_full_reload")
                {
                    let _: () = commit.call(())?;
                }
                tracing::info!(entry = %busted, "vite full reload: entry re-imported");
                Ok(())
            }
            Err(caught) => {
                if let Ok(rollback) = ctx
                    .globals()
                    .get::<_, Function>("__wabou_hmr_rollback_full_reload")
                    && let Err(error) = rollback.call::<_, ()>(())
                {
                    tracing::error!(?error, "failed to restore last-good HMR records");
                }
                tracing::error!(entry = %busted, error = %caught, "vite full reload failed");
                Err(caught.throw(ctx))
            }
        }
    }
}

impl ViteModuleCache {
    pub(crate) fn insert(&self, url: String, source: String) {
        self.sources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(url, source);
    }

    fn get(&self, url: &str) -> Option<String> {
        self.sources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(url)
            .cloned()
    }

    /// Drop every cached module source (in-process full reload).
    pub(crate) fn clear(&self) {
        self.sources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

pub(crate) struct ViteLoader {
    http: ureq::Agent,
    cache: ViteModuleCache,
}

impl ViteLoader {
    pub(crate) fn new(cache: ViteModuleCache) -> Self {
        Self {
            http: vite_http_agent(),
            cache,
        }
    }
}

fn vite_http_agent() -> ureq::Agent {
    // Vite is an application-local transport. Inheriting HTTP_PROXY here can
    // route loopback module requests through a corporate or development proxy,
    // which commonly rejects CONNECT requests to 127.0.0.1.
    ureq::Agent::config_builder()
        .proxy(None)
        .timeout_global(Some(Duration::from_secs(5)))
        .build()
        .into()
}

impl Loader for ViteLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> Result<Module<'js>> {
        let source = if Url::parse(name)
            .ok()
            .is_some_and(|url| url.path() == "/@vite/client")
        {
            HMR_CLIENT.to_owned()
        } else if let Some(source) = self.cache.get(name) {
            source
        } else {
            let mut response = self
                .http
                .get(name)
                .call()
                .map_err(|error| Error::new_loading_message(name, error.to_string()))?;
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default();
            if !content_type.contains("javascript") {
                return Err(Error::new_loading_message(
                    name,
                    format!("expected JavaScript from Vite, received {content_type:?}"),
                ));
            }
            let source = response
                .body_mut()
                .read_to_string()
                .map_err(|error| Error::new_loading_message(name, error.to_string()))?;
            self.cache.insert(name.to_owned(), source.clone());
            source
        };
        let module = Module::declare(ctx.clone(), name, source)?;
        module.meta()?.set("url", name)?;
        Ok(module)
    }
}

// ---------------------------------------------------------------------------
// HMR client — connects to Vite's WebSocket, fetches updated modules on
// notification, and forwards them to the Applier via ReloadHandle.
// ---------------------------------------------------------------------------

use serde::Deserialize;
use snafu::{ResultExt, Snafu};
use tungstenite::client::IntoClientRequest;

#[derive(Debug, Snafu)]
/// Failure while connecting to or loading updates from a Vite development server.
pub enum ViteError {
    /// The configured Vite server URL could not be parsed.
    #[snafu(display("invalid Vite server URL: {source}"))]
    InvalidUrl {
        /// URL parser failure.
        source: url::ParseError,
    },

    /// The Vite URL could not be converted to a WebSocket URL.
    #[snafu(display("invalid Vite WebSocket scheme"))]
    InvalidScheme,

    /// The HMR WebSocket could not be created or used.
    #[snafu(display("WebSocket connection failed: {source}"))]
    WebSocket {
        /// Underlying WebSocket failure.
        source: tungstenite::Error,
    },

    /// A development module or stylesheet could not be fetched.
    #[snafu(display("HTTP request to Vite server failed: {source}"))]
    Http {
        /// Underlying HTTP failure.
        source: ureq::Error,
    },

    /// Vite returned a resource with a content type that does not match its use.
    #[snafu(display("Vite returned unexpected content-type: expected {expected}, got {actual}"))]
    ContentType {
        /// Content type required by the loader.
        expected: String,
        /// Content type returned by Vite.
        actual: String,
    },

    /// The HMR protocol header could not be encoded.
    #[snafu(display("invalid header value: {source}"))]
    Header {
        /// Header validation failure.
        source: tungstenite::http::header::InvalidHeaderValue,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum VitePayload {
    Update {
        updates: Vec<ViteUpdate>,
    },
    FullReload,
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum ViteUpdate {
    JsUpdate {
        path: String,
        #[serde(rename = "acceptedPath")]
        accepted_path: String,
        timestamp: u64,
    },
    CssUpdate {
        #[serde(rename = "acceptedPath")]
        accepted_path: String,
        timestamp: u64,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, PartialEq, Eq)]
enum ViteMessage {
    Update {
        path: String,
        accepted_path: String,
        timestamp: u64,
    },
    CssUpdate {
        accepted_path: String,
    },
    FullReload,
}

/// Start a Vite HMR client on a background thread. Connects to the Vite dev
/// server's WebSocket, fetches updated module/stylesheet sources via blocking
/// HTTP, and forwards them to the Applier through `reload`. CSS notifications
/// carry only their path because Wabou styles are delivered as Style IR.
pub struct HmrClient {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for HmrClient {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Connects to a Vite server and forwards HMR updates to `reload`.
///
/// The returned handle owns the background client; dropping it requests
/// shutdown and joins the client thread.
pub fn start_hmr_client(
    server_url: &str,
    reload: crate::ReloadHandle,
) -> std::result::Result<HmrClient, ViteError> {
    let mut websocket_url = url::Url::parse(server_url).context(InvalidUrlSnafu)?;
    websocket_url
        .set_scheme(if websocket_url.scheme() == "https" {
            "wss"
        } else {
            "ws"
        })
        .map_err(|_| ViteError::InvalidScheme)?;
    let mut request = websocket_url
        .as_str()
        .into_client_request()
        .context(WebSocketSnafu)?;
    request.headers_mut().insert(
        tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL,
        "vite-hmr".parse().context(HeaderSnafu)?,
    );
    let (mut socket, _) = tungstenite::connect(request).context(WebSocketSnafu)?;
    if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_mut() {
        stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .map_err(tungstenite::Error::Io)
            .context(WebSocketSnafu)?;
    }
    let client = vite_http_agent();
    let server_url = url::Url::parse(server_url).context(InvalidUrlSnafu)?;
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();

    let thread = std::thread::spawn(move || {
        tracing::info!(url = %websocket_url, "Vite HMR client connected");
        while !thread_stop.load(Ordering::Acquire) {
            let message = match socket.read() {
                Ok(message) => message,
                Err(tungstenite::Error::Io(error))
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    continue;
                }
                Err(_) => break,
            };
            let Ok(text) = message.to_text() else {
                continue;
            };
            let payload = match serde_json::from_str::<VitePayload>(text) {
                Ok(payload) => payload,
                Err(error) => {
                    tracing::warn!(?error, payload = text, "invalid Vite HMR payload");
                    continue;
                }
            };
            for message in messages(payload) {
                let result = match message {
                    ViteMessage::Update {
                        path,
                        accepted_path,
                        timestamp,
                    } => {
                        tracing::debug!(%path, %accepted_path, timestamp, "received Vite HMR update");
                        match fetch_module(&client, &server_url, &accepted_path, timestamp) {
                            Ok(source) => reload.send(crate::ReloadMsg::HmrUpdate {
                                path,
                                accepted_path,
                                timestamp,
                                source,
                            }),
                            Err(error) => {
                                tracing::error!(%path, %accepted_path, ?error, "failed to prefetch Vite HMR module");
                                continue;
                            }
                        }
                    }
                    ViteMessage::CssUpdate { accepted_path } => {
                        reload.send(crate::ReloadMsg::CssUpdate {
                            path: accepted_path,
                        })
                    }
                    ViteMessage::FullReload => reload.send(crate::ReloadMsg::FullReload),
                };
                if result.is_err() {
                    tracing::warn!("Vite HMR receiver disconnected");
                    return;
                }
            }
        }
        if !thread_stop.load(Ordering::Acquire) {
            tracing::warn!("Vite HMR client disconnected");
        }
    });
    Ok(HmrClient {
        stop,
        thread: Some(thread),
    })
}

fn fetch_module(
    client: &ureq::Agent,
    server_url: &url::Url,
    accepted_path: &str,
    timestamp: u64,
) -> std::result::Result<String, ViteError> {
    let mut module_url = server_url
        .join(accepted_path)
        .map_err(|_| ViteError::InvalidScheme)?;
    module_url
        .query_pairs_mut()
        .append_pair("t", &timestamp.to_string());
    let mut response = client.get(module_url.as_str()).call().context(HttpSnafu)?;
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !content_type.contains("javascript") && !content_type.contains("css") {
        return Err(ViteError::ContentType {
            expected: "javascript or css".into(),
            actual: content_type.into(),
        });
    }
    response.body_mut().read_to_string().context(HttpSnafu)
}

/// Read the Vite dev server URL from the `VITE_URL` environment variable.
pub fn vite_url_from_env() -> Option<String> {
    std::env::var("VITE_URL").ok().filter(|s| !s.is_empty())
}

fn messages(payload: VitePayload) -> Vec<ViteMessage> {
    match payload {
        VitePayload::Update { updates } => updates
            .into_iter()
            .filter_map(|update| match update {
                ViteUpdate::JsUpdate {
                    path,
                    accepted_path,
                    timestamp,
                } => {
                    if accepted_path
                        .split_once('?')
                        .map_or(accepted_path.as_str(), |(path, _)| path)
                        .ends_with(".css")
                    {
                        let _ = timestamp;
                        Some(ViteMessage::CssUpdate { accepted_path })
                    } else {
                        Some(ViteMessage::Update {
                            path,
                            accepted_path,
                            timestamp,
                        })
                    }
                }
                ViteUpdate::CssUpdate {
                    accepted_path,
                    timestamp,
                } => {
                    let _ = timestamp;
                    Some(ViteMessage::CssUpdate { accepted_path })
                }
                ViteUpdate::Other => None,
            })
            .collect(),
        VitePayload::FullReload => vec![ViteMessage::FullReload],
        VitePayload::Other => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_hmr_client_is_a_valid_quickjs_module() {
        let runtime = rquickjs::AsyncRuntime::new().unwrap();
        let context =
            futures_lite::future::block_on(rquickjs::AsyncContext::full(&runtime)).unwrap();

        futures_lite::future::block_on(context.with(|ctx| {
            Module::declare(ctx, "/@vite/client", HMR_CLIENT)
                .expect("generated HMR client should compile");
        }));
    }

    #[test]
    fn resolves_vite_module_urls() {
        let resolver = ViteResolver::new(Url::parse("http://127.0.0.1:5173/").unwrap());

        assert_eq!(
            resolver
                .resolve_url("http://127.0.0.1:5173/src/App.tsx", "./Counter.tsx")
                .unwrap()
                .as_str(),
            "http://127.0.0.1:5173/src/Counter.tsx"
        );
        assert_eq!(
            resolver
                .resolve_url("http://127.0.0.1:5173/src/App.tsx", "/@solid-refresh")
                .unwrap()
                .as_str(),
            "http://127.0.0.1:5173/@solid-refresh"
        );
    }

    #[test]
    fn module_cache_returns_prefetched_source() {
        let cache = ViteModuleCache::default();
        cache.insert(
            "http://127.0.0.1:5173/src/App.tsx?t=42".to_owned(),
            "export const value = 42;".to_owned(),
        );

        assert_eq!(
            cache.get("http://127.0.0.1:5173/src/App.tsx?t=42"),
            Some("export const value = 42;".to_owned())
        );
    }

    #[test]
    fn vite_http_transport_never_inherits_an_environment_proxy() {
        assert!(vite_http_agent().config().proxy().is_none());
    }

    #[test]
    fn forwards_vite_javascript_updates() {
        let payload = serde_json::from_str(
            r#"{
                "type": "update",
                "updates": [{
                    "type": "js-update",
                    "path": "/src/Counter.tsx",
                    "acceptedPath": "/src/Counter.tsx",
                    "timestamp": 42
                }]
            }"#,
        )
        .unwrap();
        assert_eq!(
            messages(payload),
            vec![ViteMessage::Update {
                path: "/src/Counter.tsx".to_owned(),
                accepted_path: "/src/Counter.tsx".to_owned(),
                timestamp: 42,
            }]
        );
    }

    #[test]
    fn handles_css_updates_and_ignores_unknown_payloads() {
        let connected = serde_json::from_str(r#"{"type":"connected"}"#).unwrap();
        let css_update = serde_json::from_str(
            r#"{"type":"update","updates":[{
                "type":"css-update",
                "acceptedPath":"/@id/__x00__virtual:uno.css",
                "timestamp":42
            }]}"#,
        )
        .unwrap();

        assert!(messages(connected).is_empty());
        assert_eq!(
            messages(css_update),
            vec![ViteMessage::CssUpdate {
                accepted_path: "/@id/__x00__virtual:uno.css".to_owned(),
            }]
        );
    }
}
