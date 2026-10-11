//! Replaceable application behavior for the component gallery.
use gallery_api::{CAPABILITY, DESCRIBE_PALETTE, DescribePaletteRequest, DescribePaletteResponse};
use wabou_capability_sdk::export_handlers;

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

export_handlers! {
    version: CAPABILITY.version(),
    DESCRIBE_PALETTE => |_, request| describe_palette(request),
}
