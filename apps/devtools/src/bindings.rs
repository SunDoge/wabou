use serde::{Deserialize, Serialize};
use specta::Type;
use wabou_bindgen::{Bindings, Capability};
use wabou_devtools::{DebugFrame, DebugNode, DebugOverlay, DebugStatus};

/// Host capability containing the DevTools example endpoints.
pub const CAPABILITY: &str = "devtools";

/// Filesystem path returned by socket and screenshot operations.
#[derive(Clone, Debug, Deserialize, Serialize, Type)]
pub struct PathResult {
    /// Native path encoded as a platform string.
    pub path: String,
}

#[allow(unused_variables)]
mod contract {
    use super::*;

    macro_rules! endpoint {
        ($name:ident() -> $response:ty) => {
            #[specta::specta]
            pub async fn $name() -> Result<$response, String> {
                unreachable!("binding contract functions are not invoked")
            }
        };
        ($name:ident($($argument:ident: $request:ty),+ $(,)?) -> $response:ty) => {
            #[specta::specta]
            pub async fn $name($($argument: $request),+) -> Result<$response, String> {
                unreachable!("binding contract functions are not invoked")
            }
        };
    }

    endpoint!(connect(path: String) -> PathResult);
    endpoint!(status() -> DebugStatus);
    endpoint!(query_nodes(query: String, limit: u32) -> Vec<DebugNode>);
    endpoint!(inspect_node(id: u32) -> DebugNode);
    endpoint!(recent_frames(limit: u32) -> Vec<DebugFrame>);
    endpoint!(capture_screenshot() -> PathResult);
    endpoint!(set_overlay(
        layout: bool,
        clips: bool,
        hit_target: bool,
        selected_node: Option<u32>,
    ) -> DebugOverlay);
}

/// Builds the binding manifest consumed by TypeScript code generation.
pub fn manifest() -> Bindings {
    Bindings::new().capability(Capability::from_specta(
        CAPABILITY,
        specta::functions::collect_types![
            contract::connect,
            contract::status,
            contract::query_nodes,
            contract::inspect_node,
            contract::recent_frames,
            contract::capture_screenshot,
            contract::set_overlay,
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_devtools_from_rust_functions() {
        let output = manifest().render();
        assert!(output.contains("queryNodes(query: string, limit: number): Promise<DebugNode[]>"));
        assert!(output.contains("selectedNode?: number | null"));
        assert!(output.contains("captureScreenshot(): Promise<PathResult>"));
    }
}
