use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use syn::{Data, DeriveInput, Expr, Fields, LitStr, spanned::Spanned};

pub(super) fn expand(item: &DeriveInput) -> syn::Result<TokenStream> {
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new(
            item.generics.span(),
            "RoutePath requires a non-generic enum",
        ));
    }
    let Data::Enum(data) = &item.data else {
        return Err(syn::Error::new(item.span(), "RoutePath requires an enum"));
    };
    let mut patterns = BTreeSet::new();
    let mut encoders = Vec::new();
    let mut parsers = Vec::new();
    let mut formats = Vec::new();
    for variant in &data.variants {
        let mut path: Option<LitStr> = None;
        let mut guard: Option<Expr> = None;
        for attr in variant
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("route"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("path") && path.is_none() {
                    path = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("guard") && guard.is_none() {
                    guard = Some(meta.value()?.parse()?);
                } else {
                    return Err(meta.error("expected one path and optional guard"));
                }
                Ok(())
            })?;
        }
        let path = path.ok_or_else(|| {
            syn::Error::new(
                variant.span(),
                "each variant requires #[route(path = \"/...\")]",
            )
        })?;
        let value = path.value();
        if !value.starts_with('/')
            || value.len() > 4096
            || value
                .chars()
                .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '?' | '#' | '%'))
            || (value != "/" && value.ends_with('/'))
            || value.contains("//")
        {
            return Err(syn::Error::new(
                path.span(),
                "route must be an absolute canonical path without URL escapes, query, fragment or empty segments",
            ));
        }
        if !patterns.insert(value.clone()) {
            return Err(syn::Error::new(path.span(), "duplicate route path"));
        }
        let fields: Vec<_> = match &variant.fields {
            Fields::Unit => Vec::new(),
            Fields::Named(fields) => fields.named.iter().collect(),
            Fields::Unnamed(_) => {
                return Err(syn::Error::new(
                    variant.fields.span(),
                    "route parameters require named fields",
                ));
            }
        };
        for field in &fields {
            if field
                .ident
                .as_ref()
                .unwrap()
                .to_string()
                .starts_with("__actor_route_")
            {
                return Err(syn::Error::new(
                    field.span(),
                    "__actor_route_ is reserved for generated route bindings",
                ));
            }
        }
        let mut used = BTreeSet::new();
        let mut writes = Vec::new();
        let mut reads = Vec::new();
        for segment in value.split('/').skip(1) {
            if let Some(parameter) = segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                let field = fields
                    .iter()
                    .find(|field| field.ident.as_ref().unwrap() == parameter)
                    .ok_or_else(|| {
                        syn::Error::new(path.span(), "route placeholder must name a variant field")
                    })?;
                if !used.insert(parameter.to_owned()) {
                    return Err(syn::Error::new(
                        path.span(),
                        "each route parameter must occur once",
                    ));
                }
                let name = &field.ident;
                let ty = &field.ty;
                writes.push(quote! {
                    __actor_route_path.push('/');
                    __actor_route_path.push_str(&::blockui::route_codec::encode_segment(
                        &::std::string::ToString::to_string(#name)
                    ));
                });
                reads.push(quote! {
                    let __actor_route_segment = __actor_route_segments.next()?;
                    let __actor_route_segment = ::blockui::route_codec::decode_segment(__actor_route_segment)?;
                    let #name: #ty = <#ty as ::std::str::FromStr>::from_str(&__actor_route_segment).ok()?;
                });
            } else {
                if segment.contains(['{', '}']) || segment == "." || segment == ".." {
                    return Err(syn::Error::new(
                        path.span(),
                        "placeholders must occupy a whole segment; dot segments are unsupported",
                    ));
                }
                writes.push(
                    quote! { __actor_route_path.push('/'); __actor_route_path.push_str(#segment); },
                );
                reads.push(
                    quote! { if __actor_route_segments.next()? != #segment { return None; } },
                );
            }
        }
        if used.len() != fields.len() {
            return Err(syn::Error::new(
                variant.fields.span(),
                "each variant field needs a path placeholder",
            ));
        }
        let name = &variant.ident;
        let names: Vec<_> = fields.iter().map(|field| &field.ident).collect();
        let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();
        let is_default = variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("default"));
        formats.push(format!(
            "{}|{}|{}|{}",
            value,
            quote!(#(#types),*),
            quote!(#guard),
            is_default
        ));
        let pattern = if matches!(variant.fields, Fields::Unit) {
            quote!(Self::#name)
        } else {
            quote!(Self::#name { #(#names),* })
        };
        let guard = guard.map(|guard| quote! { if !(#guard) { return None; } });
        encoders.push(quote! { #pattern => { let mut __actor_route_path = ::std::string::String::new(); #(#writes)* __actor_route_path } });
        parsers.push(quote! {
            if let Some(__actor_route_candidate) = (|| -> ::std::option::Option<Self> {
                let mut __actor_route_segments = __actor_route_input.split('/');
                if __actor_route_segments.next()? != "" { return None; }
                #(#reads)*
                if __actor_route_segments.next().is_some() { return None; }
                #guard
                let __actor_route_candidate = #pattern;
                if <Self as ::blockui::RoutePath>::to_path(&__actor_route_candidate) != __actor_route_input { return None; }
                Some(__actor_route_candidate)
            })() {
                // Ambiguous overlapping patterns are rejected, never resolved by source order.
                if __actor_route_match.is_some() { return None; }
                __actor_route_match = Some(__actor_route_candidate);
            }
        });
    }
    formats.sort();
    let mut hash = 0xcbf29ce484222325_u64;
    for part in std::iter::once("percent-utf8-v1").chain(formats.iter().map(String::as_str)) {
        for byte in (part.len() as u64)
            .to_le_bytes()
            .into_iter()
            .chain(part.bytes())
        {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    let format = format!("@route1-{hash:016x}");
    let name = &item.ident;
    Ok(quote! {
        impl ::blockui::RoutePath for #name {
            const FORMAT: &'static str = #format;
            fn to_path(&self) -> ::std::string::String {
                match self { #(#encoders),* }
            }
            fn from_path(__actor_route_input: &str) -> ::std::option::Option<Self> {
                if __actor_route_input.len() > 4096 || __actor_route_input.chars().any(|c| c.is_control() || c.is_whitespace() || matches!(c, '?' | '#')) { return None; }
                let mut __actor_route_match = None;
                #(#parsers)*
                __actor_route_match
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_route_declarations_are_diagnosed_before_codegen() {
        for source in [
            "struct Route;",
            "enum Route<T> { Home }",
            "enum Route { Home }",
            r#"enum Route { #[route(path = "relative")] Home }"#,
            r#"enum Route { #[route(path = "/x/")] Home }"#,
            r#"enum Route { #[route(path = "/x//y")] Home }"#,
            r#"enum Route { #[route(path = "/{missing}")] Home }"#,
            r#"enum Route { #[route(path = "/{id}/{id}")] Item { id: u64 } }"#,
            r#"enum Route { #[route(path = "/")] Item { id: u64 } }"#,
            r#"enum Route { #[route(path = "/{id}")] Item(u64) }"#,
            r#"enum Route { #[route(path = "/prefix-{id}")] Item { id: u64 } }"#,
            r#"enum Route { #[route(path = "/")] A, #[route(path = "/")] B }"#,
            r#"enum Route { #[route(path = "/{__actor_route_path}")] Item { __actor_route_path: u64 } }"#,
        ] {
            let item: DeriveInput = syn::parse_str(source).unwrap();
            assert!(expand(&item).is_err(), "{source}");
        }
    }
}
