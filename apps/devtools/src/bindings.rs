use serde::{Deserialize, Serialize};
use wabou::{Bindings, Capability, JsonCapabilityContract, JsonMethod, Type, specta};
use wabou_devtools::{DebugFrame, DebugNode, DebugOverlay, DebugStatus, NodeKey};

/// Host capability containing the DevTools example endpoints.
pub const CAPABILITY: JsonCapabilityContract = JsonCapabilityContract::new("devtools", 1);

/// Wire name of each DevTools endpoint.
pub mod method {
    use super::*;

    /// Select an inspected runtime socket.
    pub const CONNECT: JsonMethod<ConnectRequest, PathResult> = JsonMethod::new("connect");
    /// Read inspected runtime status.
    pub const STATUS: JsonMethod<(), DebugStatus> = JsonMethod::no_request("status");
    /// Query retained nodes.
    pub const QUERY_NODES: JsonMethod<QueryNodesRequest, Vec<DebugNode>> =
        JsonMethod::new("queryNodes");
    /// Inspect one retained node.
    pub const INSPECT_NODE: JsonMethod<InspectNodeRequest, DebugNode> =
        JsonMethod::new("inspectNode");
    /// Read recent bridge frames.
    pub const RECENT_FRAMES: JsonMethod<RecentFramesRequest, Vec<DebugFrame>> =
        JsonMethod::new("recentFrames");
    /// Capture a native screenshot.
    pub const CAPTURE_SCREENSHOT: JsonMethod<(), PathResult> =
        JsonMethod::no_request("captureScreenshot");
    /// Configure the inspected runtime overlay.
    pub const SET_OVERLAY: JsonMethod<SetOverlayRequest, DebugOverlay> =
        JsonMethod::new("setOverlay");
}

/// Filesystem path returned by socket and screenshot operations.
#[derive(Clone, Debug, Deserialize, Serialize, Type)]
pub struct PathResult {
    /// Native path encoded as a platform string.
    pub path: String,
}

/// Request selecting the inspected runtime socket.
#[derive(Deserialize, Type)]
pub struct ConnectRequest {
    /// Native DevTools socket path.
    pub path: String,
}

/// Request filtering the retained node tree.
#[derive(Deserialize, Type)]
pub struct QueryNodesRequest {
    /// Text matched against node metadata.
    pub query: String,
    /// Maximum number of results.
    pub limit: u32,
}

/// Request selecting one retained node.
#[derive(Deserialize, Type)]
pub struct InspectNodeRequest {
    /// Retained node identifier.
    pub id: NodeKey,
}

/// Request selecting recent protocol frames.
#[derive(Deserialize, Type)]
pub struct RecentFramesRequest {
    /// Maximum number of frames.
    pub limit: u32,
}

/// Request configuring the inspected runtime overlay.
#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetOverlayRequest {
    /// Show layout boxes.
    pub layout: bool,
    /// Show clip chains.
    pub clips: bool,
    /// Show the current hit target.
    pub hit_target: bool,
    /// Optionally highlight one retained node.
    pub selected_node: Option<NodeKey>,
}

/// Builds the binding manifest consumed by TypeScript code generation.
pub fn manifest() -> Bindings {
    Bindings::new().capability(
        Capability::new(CAPABILITY)
            .method(method::CONNECT)
            .method(method::STATUS)
            .method(method::QUERY_NODES)
            .method(method::INSPECT_NODE)
            .method(method::RECENT_FRAMES)
            .method(method::CAPTURE_SCREENSHOT)
            .method(method::SET_OVERLAY),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_stateful_devtools_methods_from_dto_contracts() {
        let output = manifest().render();
        assert!(output.contains("queryNodes(request: QueryNodesRequest): Promise<DebugNode[]>"));
        assert!(output.contains("selectedNode: NodeKey | null"));
        assert!(output.contains("captureScreenshot(): Promise<PathResult>"));
        assert!(output.contains("captureScreenshot(): NativeResult<string>"));
    }
}
