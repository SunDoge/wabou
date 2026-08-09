use serde::{Deserialize, Serialize};
use specta::Type;
use wabou_bindings::{Bindings, Capability};

pub const CAPABILITY: &str = "bindingsDemo";
pub const DESCRIBE_PALETTE: &str = "describePalette";

#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DescribePaletteRequest {
    pub name: String,
    pub swatch_count: u32,
}

#[derive(Serialize, Type)]
#[serde(tag = "status", rename_all = "camelCase")]
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
    Bindings::new().capability(Capability::from_specta(
        CAPABILITY,
        specta::functions::collect_types![describe_palette],
    ))
}

#[specta::specta]
pub async fn describe_palette(
    request: DescribePaletteRequest,
) -> Result<DescribePaletteResponse, String> {
    Ok(DescribePaletteResponse::Palette {
        title: format!("{} palette", request.name),
        swatches: (1..=request.swatch_count.min(8))
            .map(|index| format!("{}-{index}", request.name))
            .collect(),
    })
}

pub async fn invoke_describe_palette(raw: &str) -> String {
    let result = match serde_json::from_str::<DescribePaletteRequest>(raw) {
        Ok(request) => describe_palette(request).await,
        Err(error) => Err(format!("invalid palette request: {error}")),
    };
    match result {
        Ok(value) => serde_json::json!({ "ok": true, "value": value }).to_string(),
        Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_method_and_manifest_share_names() {
        let output = manifest().render();
        assert!(output.contains(&format!("readonly {CAPABILITY}")));
        assert!(output.contains(&format!("{DESCRIBE_PALETTE}(request: string)")));
        assert!(output.contains(
            "describePalette(request: DescribePaletteRequest): Promise<DescribePaletteResponse>"
        ));
    }
}
