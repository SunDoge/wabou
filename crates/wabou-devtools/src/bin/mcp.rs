//! MCP adapter for the local Wabou DevTools socket.

use std::path::PathBuf;

use base64::Engine as _;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router, transport::stdio};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wabou_devtools::{call, discover_socket, empty_params, request};

#[derive(Clone)]
struct WabouMcp {
    path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct QueryParams {
    query: String,
    #[serde(default = "default_query_limit")]
    limit: usize,
}

fn default_query_limit() -> usize {
    100
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NodeParams {
    id: McpNodeKey,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
struct McpNodeKey {
    lo: u32,
    hi: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PointParams {
    x: f32,
    y: f32,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FramesParams {
    #[serde(default = "default_frame_limit")]
    limit: usize,
}

fn default_frame_limit() -> usize {
    20
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlayParams {
    layout: bool,
    clips: bool,
    hit_target: bool,
    selected_node: Option<McpNodeKey>,
}

#[derive(Debug, Default, Deserialize, JsonSchema, Serialize)]
struct CaptureParams {
    x: Option<f32>,
    y: Option<f32>,
}

impl WabouMcp {
    async fn runtime_call(&self, method: &'static str, params: Value) -> Result<Value, String> {
        let configured = self.path.clone();
        tokio::task::spawn_blocking(move || {
            let path = configured.map_or_else(discover_socket, Ok)?;
            let response = call(&path, &request(1, method, params))?;
            if let Some(error) = response.error {
                return Err(error);
            }
            Ok(response.result.unwrap_or(Value::Null))
        })
        .await
        .map_err(|error| format!("DevTools task failed: {error}"))?
    }

    fn tool_result(result: Result<Value, String>, image_field: Option<&str>) -> CallToolResult {
        match result {
            Ok(value) => {
                let mut content = Vec::new();
                if let Some(field) = image_field {
                    let path = value.get(field).and_then(Value::as_str);
                    match path
                        .ok_or_else(|| format!("response did not contain {field}"))
                        .and_then(|path| {
                            std::fs::read(path)
                                .map_err(|error| format!("cannot read screenshot: {error}"))
                        }) {
                        Ok(png) => content.push(ContentBlock::image(
                            base64::engine::general_purpose::STANDARD.encode(png),
                            "image/png",
                        )),
                        Err(error) => {
                            return CallToolResult::error(vec![ContentBlock::text(error)]);
                        }
                    }
                }
                content.push(ContentBlock::json(&value).unwrap_or_else(|error| {
                    ContentBlock::text(format!("cannot serialize tool result: {error}"))
                }));
                let mut result = CallToolResult::structured(value);
                result.content = content;
                result
            }
            Err(error) => CallToolResult::error(vec![ContentBlock::text(error)]),
        }
    }
}

#[tool_router]
impl WabouMcp {
    #[tool(
        name = "wabou_status",
        description = "Get runtime, viewport and tree revision"
    )]
    async fn status(&self) -> CallToolResult {
        Self::tool_result(self.runtime_call("status", empty_params()).await, None)
    }

    #[tool(
        name = "wabou_query_nodes",
        description = "Find nodes by tag, text or class"
    )]
    async fn query_nodes(&self, Parameters(params): Parameters<QueryParams>) -> CallToolResult {
        Self::tool_result(
            self.runtime_call(
                "queryNodes",
                json!({"query": params.query, "limit": params.limit}),
            )
            .await,
            None,
        )
    }

    #[tool(
        name = "wabou_inspect_node",
        description = "Inspect structure, layout, computed style and clip coordinates for a node"
    )]
    async fn inspect_node(&self, Parameters(params): Parameters<NodeParams>) -> CallToolResult {
        Self::tool_result(
            self.runtime_call("inspectNode", json!({"id": params.id}))
                .await,
            None,
        )
    }

    #[tool(
        name = "wabou_inspect_at_point",
        description = "Find the topmost hit-testable node at logical window coordinates and return its ancestors"
    )]
    async fn inspect_at_point(
        &self,
        Parameters(params): Parameters<PointParams>,
    ) -> CallToolResult {
        Self::tool_result(
            self.runtime_call("inspectAtPoint", json!({"x": params.x, "y": params.y}))
                .await,
            None,
        )
    }

    #[tool(
        name = "wabou_recent_frames",
        description = "Read recent Host-to-JS and JS-to-Host protocol frames"
    )]
    async fn recent_frames(&self, Parameters(params): Parameters<FramesParams>) -> CallToolResult {
        Self::tool_result(
            self.runtime_call("recentFrames", json!({"limit": params.limit}))
                .await,
            None,
        )
    }

    #[tool(
        name = "wabou_validate_snapshot",
        description = "Validate retained node identity, parent chains, geometry, clips, transforms, interaction targets and semantic references"
    )]
    async fn validate_snapshot(&self) -> CallToolResult {
        Self::tool_result(
            self.runtime_call("validateSnapshot", empty_params()).await,
            None,
        )
    }

    #[tool(
        name = "wabou_set_layout_overlay",
        description = "Show or hide native layout, clip and hit-test diagnostics"
    )]
    async fn set_layout_overlay(
        &self,
        Parameters(params): Parameters<OverlayParams>,
    ) -> CallToolResult {
        Self::tool_result(
            self.runtime_call(
                "setOverlay",
                serde_json::to_value(params).expect("overlay params serialize"),
            )
            .await,
            None,
        )
    }

    #[tool(
        name = "wabou_capture_screenshot",
        description = "Capture the current window as a PNG"
    )]
    async fn capture_screenshot(&self) -> CallToolResult {
        Self::tool_result(
            self.runtime_call("captureScreenshot", empty_params()).await,
            Some("path"),
        )
    }

    #[tool(
        name = "wabou_capture_case",
        description = "Atomically capture a screenshot, runtime snapshot, recent frames and optional point inspection from one rendered frame"
    )]
    async fn capture_case(&self, Parameters(params): Parameters<CaptureParams>) -> CallToolResult {
        let params = match (params.x, params.y) {
            (Some(x), Some(y)) => json!({"x": x, "y": y}),
            (None, None) => empty_params(),
            _ => {
                return CallToolResult::error(vec![ContentBlock::text(
                    "x and y must be provided together",
                )]);
            }
        };
        Self::tool_result(
            self.runtime_call("captureCase", params).await,
            Some("screenshotPath"),
        )
    }
}

#[tool_handler(name = "wabou-devtools")]
impl ServerHandler for WabouMcp {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = WabouMcp {
        path: std::env::var_os("WABOU_DEVTOOLS_SOCKET").map(PathBuf::from),
    };
    server.serve(stdio()).await?.waiting().await?;
    Ok(())
}
