//! Shared palette DTOs, with no runtime dependency.

use serde::{Deserialize, Serialize};

/// Input accepted by the palette-description example.
#[derive(Deserialize, Serialize)]
#[wabou_bindgen::js_type(fields(name = string, swatchCount = number))]
#[serde(rename_all = "camelCase")]
pub struct DescribePaletteRequest {
    /// Base palette name.
    pub name: String,
    /// Requested number of example swatches.
    pub swatch_count: u32,
}

/// Result returned by the palette-description example.
#[derive(Deserialize, Serialize)]
#[wabou_bindgen::js_type(fields(title = string, swatches = array(string)))]
#[serde(rename_all = "camelCase")]
pub struct DescribePaletteResponse {
    /// Human-readable palette title.
    pub title: String,
    /// Generated swatch token names.
    pub swatches: Vec<String>,
}
