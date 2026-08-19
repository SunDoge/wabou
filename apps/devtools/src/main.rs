//! Native host executable for the Wabou DevTools example.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};
use wabou_devtools::{
    DebugFrame, DebugNode, DebugOverlay, DebugStatus, call, discover_socket, empty_params, request,
};
use wabou_devtools_app::bindings::{
    CAPABILITY, ConnectRequest, InspectNodeRequest, PathResult, QueryNodesRequest,
    RecentFramesRequest, SetOverlayRequest, method,
};

type Target = Arc<Mutex<Option<PathBuf>>>;

fn resolve_target(target: &Target) -> Result<PathBuf, String> {
    target
        .lock()
        .map_err(|_| "target selection lock poisoned".to_string())?
        .clone()
        .map_or_else(discover_socket, Ok)
}

async fn rpc<Response: DeserializeOwned>(
    target: Target,
    method: &'static str,
    params: Value,
) -> Result<Response, String> {
    let value = tokio::task::spawn_blocking(move || {
        resolve_target(&target).and_then(|path| {
            let response = call(&path, &request(1, method, params))?;
            response
                .error
                .map_or_else(|| Ok(response.result.unwrap_or(Value::Null)), Err)
        })
    })
    .await
    .map_err(|error| format!("DevTools task failed: {error}"))??;
    serde_json::from_value(value).map_err(|error| format!("invalid DevTools response: {error}"))
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
        .json_capability(CAPABILITY, move |capability| {
            let connect_target = target.clone();
            capability.method(method::CONNECT, move |connection: ConnectRequest| {
                let target = connect_target.clone();
                async move {
                    let path = PathBuf::from(connection.path);
                    let validation = tokio::task::spawn_blocking({
                        let path = path.clone();
                        move || call(&path, &request(1, "status", empty_params()))
                    })
                    .await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result.map_err(|error| error.to_string()));
                    match validation {
                        Ok(response) if response.error.is_none() => {
                            target
                                .lock()
                                .map_err(|_| "target selection lock poisoned".to_owned())?
                                .replace(path.clone());
                            Ok(PathResult {
                                path: path.to_string_lossy().into_owned(),
                            })
                        }
                        Ok(response) => Err(response.error.unwrap()),
                        Err(error) => Err(error),
                    }
                }
            })?;

            let status_target = target.clone();
            capability.method(method::STATUS, move |(): ()| {
                rpc::<DebugStatus>(status_target.clone(), "status", empty_params())
            })?;
            let query_target = target.clone();
            capability.method(method::QUERY_NODES, move |request: QueryNodesRequest| {
                rpc::<Vec<DebugNode>>(
                    query_target.clone(),
                    "queryNodes",
                    json!({"query": request.query, "limit": request.limit}),
                )
            })?;
            let inspect_target = target.clone();
            capability.method(method::INSPECT_NODE, move |request: InspectNodeRequest| {
                rpc::<DebugNode>(
                    inspect_target.clone(),
                    "inspectNode",
                    json!({"id": request.id}),
                )
            })?;
            let frames_target = target.clone();
            capability.method(
                method::RECENT_FRAMES,
                move |request: RecentFramesRequest| {
                    rpc::<Vec<DebugFrame>>(
                        frames_target.clone(),
                        "recentFrames",
                        json!({"limit": request.limit}),
                    )
                },
            )?;
            let screenshot_target = target.clone();
            let overlay_target = target.clone();
            capability.method(method::SET_OVERLAY, move |request: SetOverlayRequest| {
                rpc::<DebugOverlay>(
                    overlay_target.clone(),
                    "setOverlay",
                    json!({
                        "layout": request.layout,
                        "clips": request.clips,
                        "hitTarget": request.hit_target,
                        "selectedNode": request.selected_node,
                    }),
                )
            })?;
            capability.method(method::CAPTURE_SCREENSHOT, move |(): ()| {
                rpc::<PathResult>(
                    screenshot_target.clone(),
                    "captureScreenshot",
                    empty_params(),
                )
            })
        })
        .run()
        .whatever_context("failed to run Wabou DevTools")
}
