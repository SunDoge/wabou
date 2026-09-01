use serde::{Deserialize, Serialize};
use wabou::{Bindings, Capability, CapabilityContract, JsonMethod, Type, specta};

/// Capability containing the gallery's binding demonstration.
pub const CAPABILITY: CapabilityContract = CapabilityContract::new("bindingsDemo", 1);
/// Input accepted by the palette-description example.
#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DescribePaletteRequest {
    /// Base palette name.
    pub name: String,
    /// Requested number of example swatches.
    pub swatch_count: u32,
}

/// Result returned by the palette-description example.
#[derive(Serialize, Type)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum DescribePaletteResponse {
    /// A generated palette description.
    Palette {
        /// Human-readable palette title.
        title: String,
        /// Generated swatch token names.
        swatches: Vec<String>,
    },
    /// A rejected palette request.
    Error {
        /// Human-readable error description.
        message: String,
    },
}

/// Typed contract of the palette-description endpoint.
pub const DESCRIBE_PALETTE: JsonMethod<DescribePaletteRequest, DescribePaletteResponse> =
    JsonMethod::new("describePalette");

/// Builds the gallery's typed binding manifest.
pub fn manifest() -> Bindings {
    Bindings::new().capability(Capability::new(CAPABILITY).method(DESCRIBE_PALETTE))
}

/// Generates a small palette description for the binding demonstration.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_method_and_manifest_share_names() {
        let output = manifest().render();
        assert!(output.contains(&format!("readonly {}", CAPABILITY.name())));
        assert!(output.contains(&format!("{}(request: string)", DESCRIBE_PALETTE.name())));
        assert!(output.contains(
            "describePalette(request: DescribePaletteRequest): Promise<DescribePaletteResponse>"
        ));
    }
}
