//! Replaceable application behavior for the component gallery.
use gallery_api::{DescribePaletteRequest, DescribePaletteResponse};

/// Generates a small palette description for the binding demonstration.
pub fn describe_palette(
    request: DescribePaletteRequest,
) -> Result<DescribePaletteResponse, String> {
    Ok(DescribePaletteResponse::Palette {
        title: format!("{} palette", request.name),
        swatches: (1..=request.swatch_count.min(8))
            .map(|index| format!("{}-{index}", request.name))
            .collect(),
    })
}
