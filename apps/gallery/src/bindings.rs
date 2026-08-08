use serde::{Deserialize, Serialize};
use wabou_bindings::{Bindings, Capability, Method, TS};

pub const CAPABILITY: &str = "bindingsDemo";
pub const DESCRIBE_PALETTE: &str = "describePalette";

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(crate = "wabou_bindings::ts_rs", rename_all = "camelCase")]
pub struct DescribePaletteRequest {
    pub name: String,
    pub swatch_count: u32,
}

#[derive(Serialize, TS)]
#[serde(tag = "status", rename_all = "camelCase")]
#[ts(
    crate = "wabou_bindings::ts_rs",
    tag = "status",
    rename_all = "camelCase"
)]
pub enum DescribePaletteResponse {
    Palette {
        title: String,
        swatches: Vec<String>,
    },
    Error {
        message: String,
    },
}

pub fn manifest() -> Bindings {
    Bindings::new().capability(Capability::new(CAPABILITY).method(Method::json::<
        DescribePaletteRequest,
        DescribePaletteResponse,
    >(DESCRIBE_PALETTE)))
}

pub fn describe_palette(raw: &str) -> String {
    let response = match serde_json::from_str::<DescribePaletteRequest>(raw) {
        Ok(request) => DescribePaletteResponse::Palette {
            title: format!("{} palette", request.name),
            swatches: (1..=request.swatch_count.min(8))
                .map(|index| format!("{}-{index}", request.name))
                .collect(),
        },
        Err(error) => DescribePaletteResponse::Error {
            message: format!("invalid palette request: {error}"),
        },
    };
    serde_json::to_string(&response).expect("palette response is serializable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_method_and_manifest_share_names() {
        let output = manifest().render();
        assert!(output.contains(&format!("readonly {CAPABILITY}")));
        assert!(output.contains(&format!("{DESCRIBE_PALETTE}(request: string)")));
    }
}
