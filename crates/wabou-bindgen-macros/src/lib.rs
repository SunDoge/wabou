//! Inert source markers consumed by `wabou bindings`.

use proc_macro::TokenStream;

/// Mark a Rust type as a JavaScript class declaration source.
///
/// The macro intentionally leaves the item unchanged. The Wabou CLI reads the
/// authored attribute from source and validates its contract separately.
#[proc_macro_attribute]
pub fn js_class(_contract: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Describe the JavaScript signature of an exported Rust method.
///
/// The macro intentionally leaves the item unchanged. This permits it to sit
/// beside `#[qjs(...)]` inside an `#[rquickjs::methods]` implementation.
/// Names and runtime behavior continue to come from `rquickjs`; this marker
/// only supplies argument and return types to `wabou bindings`.
#[proc_macro_attribute]
pub fn js(_contract: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Describe a TypeScript data declaration owned by a Rust type.
///
/// Supported source shapes are explicit object fields, string-value unions,
/// and aliases. The CLI validates the authored contract and emits the type.
#[proc_macro_attribute]
pub fn js_type(_contract: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Add TypeScript metadata to an `#[rquickjs::module]` Rust module.
///
/// Public `#[rquickjs::function]` items carrying [`js`] become members of the
/// generated TypeScript host contract. `rquickjs` remains solely responsible
/// for generating and registering the runtime module.
#[proc_macro_attribute]
pub fn js_module(_contract: TokenStream, item: TokenStream) -> TokenStream {
    item
}
