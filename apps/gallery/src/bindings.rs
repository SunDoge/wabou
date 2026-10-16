//! Rust-owned gallery host functions and their TypeScript contract.
pub use gallery_api::*;
use wabou::{JsValue, rquickjs};

#[wabou::js_module(name = "bindingsDemo")]
#[rquickjs::module(rename_vars = "camelCase")]
pub mod bindings_demo {
    use super::*;

    /// Generate a small named palette.
    #[rquickjs::function]
    #[qjs(rename = "describePalette")]
    #[wabou::js(
        args(request = named(DescribePaletteRequest)),
        returns = named(DescribePaletteResponse)
    )]
    pub fn describe_palette(
        JsValue(request): JsValue<DescribePaletteRequest>,
    ) -> rquickjs::Result<JsValue<DescribePaletteResponse>> {
        gallery_functions::describe_palette(request)
            .map(JsValue)
            .map_err(|error| {
                rquickjs::Error::new_into_js_message(
                    "gallery host function",
                    "DescribePaletteResponse",
                    error,
                )
            })
    }
}
