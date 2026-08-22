use serde::{Deserialize, Serialize};
use wabou::{Bindings, Capability, JsonCapabilityContract, JsonMethod, Type, specta};
use wabou_devtools::{
    DebugCaptureCase, DebugFrame, DebugNode, DebugOverlay, DebugPointInspection, DebugStatus,
    DebugValidationReport, NodeKey,
};

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
    /// Hit-test one logical point in the inspected viewport.
    pub const INSPECT_AT_POINT: JsonMethod<InspectPointRequest, DebugPointInspection> =
        JsonMethod::new("inspectAtPoint");
    /// Read recent bridge frames.
    pub const RECENT_FRAMES: JsonMethod<RecentFramesRequest, Vec<DebugFrame>> =
        JsonMethod::new("recentFrames");
    /// Validate the current retained snapshot.
    pub const VALIDATE_SNAPSHOT: JsonMethod<(), DebugValidationReport> =
        JsonMethod::no_request("validateSnapshot");
    /// Capture a native screenshot.
    pub const CAPTURE_SCREENSHOT: JsonMethod<(), PathResult> =
        JsonMethod::no_request("captureScreenshot");
    /// Atomically capture pixels, retained state, frames and an optional point hit-test.
    pub const CAPTURE_CASE: JsonMethod<CaptureCaseRequest, DebugCaptureCase> =
        JsonMethod::new("captureCase");
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

/// Request hit-testing one logical point in the inspected viewport.
#[derive(Deserialize, Type)]
pub struct InspectPointRequest {
    /// Horizontal logical coordinate.
    pub x: f32,
    /// Vertical logical coordinate.
    pub y: f32,
}

/// Request an atomic capture, optionally hit-testing one logical point.
#[derive(Deserialize, Type)]
pub struct CaptureCaseRequest {
    /// Horizontal logical coordinate, paired with [`Self::y`].
    pub x: Option<f32>,
    /// Vertical logical coordinate, paired with [`Self::x`].
    pub y: Option<f32>,
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
            .method(method::INSPECT_AT_POINT)
            .method(method::RECENT_FRAMES)
            .method(method::VALIDATE_SNAPSHOT)
            .method(method::CAPTURE_SCREENSHOT)
            .method(method::CAPTURE_CASE)
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
        assert!(output.contains(
            "inspectAtPoint(request: InspectPointRequest): Promise<DebugPointInspection>"
        ));
        assert!(output.contains("selectedNode: NodeKey | null"));
        assert!(output.contains("validateSnapshot(): Promise<DebugValidationReport>"));
        assert!(output.contains("captureScreenshot(): Promise<PathResult>"));
        assert!(output.contains(
            "captureCase(request: CaptureCaseRequest): Promise<DebugCaptureCase_Serialize>"
        ));
        assert!(output.contains("captureScreenshot(): NativeResult<string>"));
    }
}
