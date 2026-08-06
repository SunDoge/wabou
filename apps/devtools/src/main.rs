use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use snafu::{ResultExt, Whatever};
use wabou_devtools::{call, discover_socket, empty_params, request};
use wabou_quick::rquickjs::{Function, prelude::Async};
use wabou_quick::{HostBuilder, WindowOptions};

type Target = Arc<Mutex<Option<PathBuf>>>;

fn response_json(result: Result<Value, String>) -> String {
    match result {
        Ok(value) => json!({"ok": true, "value": value}).to_string(),
        Err(error) => json!({"ok": false, "error": error}).to_string(),
    }
}

fn resolve_target(target: &Target) -> Result<PathBuf, String> {
    target
        .lock()
        .map_err(|_| "target selection lock poisoned".to_string())?
        .clone()
        .map_or_else(discover_socket, Ok)
}

async fn rpc(target: Target, method: &'static str, params: Value) -> String {
    tokio::task::spawn_blocking(move || {
        let result = resolve_target(&target).and_then(|path| {
            let response = call(&path, &request(1, method, params))?;
            response
                .error
                .map_or_else(|| Ok(response.result.unwrap_or(Value::Null)), Err)
        });
        response_json(result)
    })
    .await
    .unwrap_or_else(|error| response_json(Err(format!("DevTools task failed: {error}"))))
}

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let target: Target = Arc::new(Mutex::new(
        std::env::var_os("WABOU_DEVTOOLS_SOCKET").map(PathBuf::from),
    ));
    HostBuilder::new()
        .window(
            WindowOptions::new()
                .title("Wabou DevTools")
                .initial_inner_size(1440, 900)
                .min_inner_size(960, 600),
        )
        .devtools(false)
        .capability("devtools", move |ctx, capability| {
            let connect_target = target.clone();
            capability.set(
                "connect",
                Function::new(
                    ctx.clone(),
                    Async(move |path: String| {
                        let target = connect_target.clone();
                        async move {
                            let path = PathBuf::from(path);
                            let validation = tokio::task::spawn_blocking({
                                let path = path.clone();
                                move || call(&path, &request(1, "status", empty_params()))
                            })
                            .await
                            .map_err(|error| error.to_string())
                            .and_then(|result| result.map_err(|error| error.to_string()));
                            match validation {
                                Ok(response) if response.error.is_none() => {
                                    if let Ok(mut selected) = target.lock() {
                                        *selected = Some(path.clone());
                                    }
                                    response_json(Ok(json!({"path": path})))
                                }
                                Ok(response) => response_json(Err(response.error.unwrap())),
                                Err(error) => response_json(Err(error)),
                            }
                        }
                    }),
                )?,
            )?;

            let status_target = target.clone();
            capability.set(
                "status",
                Function::new(
                    ctx.clone(),
                    Async(move || rpc(status_target.clone(), "status", empty_params())),
                )?,
            )?;
            let query_target = target.clone();
            capability.set(
                "queryNodes",
                Function::new(
                    ctx.clone(),
                    Async(move |query: String, limit: u32| {
                        rpc(
                            query_target.clone(),
                            "queryNodes",
                            json!({"query": query, "limit": limit}),
                        )
                    }),
                )?,
            )?;
            let inspect_target = target.clone();
            capability.set(
                "inspectNode",
                Function::new(
                    ctx.clone(),
                    Async(move |id: u32| {
                        rpc(inspect_target.clone(), "inspectNode", json!({"id": id}))
                    }),
                )?,
            )?;
            let frames_target = target.clone();
            capability.set(
                "recentFrames",
                Function::new(
                    ctx.clone(),
                    Async(move |limit: u32| {
                        rpc(
                            frames_target.clone(),
                            "recentFrames",
                            json!({"limit": limit}),
                        )
                    }),
                )?,
            )?;
            let screenshot_target = target.clone();
            let overlay_target = target.clone();
            capability.set(
                "setOverlay",
                Function::new(
                    ctx.clone(),
                    Async(
                        move |layout: bool,
                              clips: bool,
                              hit_target: bool,
                              selected_node: Option<u32>| {
                            rpc(
                                overlay_target.clone(),
                                "setOverlay",
                                json!({
                                    "layout": layout,
                                    "clips": clips,
                                    "hitTarget": hit_target,
                                    "selectedNode": selected_node,
                                }),
                            )
                        },
                    ),
                ),
            )?;
            capability.set(
                "captureScreenshot",
                Function::new(
                    ctx,
                    Async(move || {
                        rpc(
                            screenshot_target.clone(),
                            "captureScreenshot",
                            empty_params(),
                        )
                    }),
                )?,
            )?;
            Ok(())
        })
        .run()
        .whatever_context("failed to run Wabou DevTools")
}
