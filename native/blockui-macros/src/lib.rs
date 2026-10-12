//! Small declaration sugar; all expressions and widget properties remain Rust.
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Fields, GenericArgument, ItemStruct, LitStr, PathArguments, Type, parse_macro_input,
    spanned::Spanned,
};
mod function_component;
mod opaque_impl;
mod route;
#[cfg(feature = "rsx")]
mod rsx;
mod sdk_path;
mod view_blocks;

// macro_rules! `ty` fragments reach proc macros inside invisible groups.
// Normalize type wrappers for classification; the represented Rust type is unchanged.
fn normalize_type(ty: &mut Type) {
    loop {
        let inner = match ty {
            Type::Group(group) => Some((*group.elem).clone()),
            Type::Paren(paren) => Some((*paren.elem).clone()),
            _ => None,
        };
        match inner {
            Some(inner) => *ty = inner,
            None => break,
        }
    }
    if let Type::Reference(reference) = ty {
        normalize_type(&mut reference.elem);
    }
    if let Type::Tuple(tuple) = ty {
        for element in &mut tuple.elems {
            normalize_type(element);
        }
    }
    if let Type::Path(path) = ty {
        for segment in &mut path.path.segments {
            if let PathArguments::AngleBracketed(arguments) = &mut segment.arguments {
                for argument in &mut arguments.args {
                    if let GenericArgument::Type(inner) = argument {
                        normalize_type(inner);
                    }
                }
            }
        }
    }
}

/// Typed keys for collections with several node roles (e.g. row and spacer).
#[proc_macro_derive(ViewKey)]
pub fn derive_view_key(input: TokenStream) -> TokenStream {
    sdk_path::resolve(derive_view_key_tokens(input))
}
fn derive_view_key_tokens(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::DeriveInput);
    let name = &item.ident;
    let syn::Data::Enum(data) = &item.data else {
        return syn::Error::new(item.span(), "ViewKey requires an enum")
            .into_compile_error()
            .into();
    };
    if !item.generics.params.is_empty() {
        return syn::Error::new(item.span(), "ViewKey does not support generic enums")
            .into_compile_error()
            .into();
    }
    let mut arms = Vec::new();
    for variant in &data.variants {
        let id = &variant.ident;
        let base = quote!(::blockui::Key::named(concat!(
            module_path!(),
            "::",
            stringify!(#name),
            "::",
            stringify!(#id)
        )));
        match &variant.fields {
            Fields::Unit => arms.push(quote!(#name::#id => #base)),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                arms.push(quote!(#name::#id(value) => #base.scoped(::blockui::Key::from(value).0)))
            }
            _ => {
                return syn::Error::new(
                    variant.span(),
                    "ViewKey variants require zero or one key payload",
                )
                .into_compile_error()
                .into();
            }
        }
    }
    quote!(impl ::core::convert::From<#name> for ::blockui::Key {
        fn from(value: #name) -> Self { match value { #(#arms),* } }
    })
    .into()
}

/// Generate canonical full-path encoding and typed segment parsing for an enum.
#[proc_macro_derive(RoutePath, attributes(route))]
pub fn derive_route_path(input: TokenStream) -> TokenStream {
    sdk_path::resolve(derive_route_path_tokens(input))
}
fn derive_route_path_tokens(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::DeriveInput);
    match route::expand(&item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

/// Generate a bounded canonical codec for named scalar/text record fields.
#[proc_macro_derive(Record, attributes(record))]
pub fn derive_record(input: TokenStream) -> TokenStream {
    sdk_path::resolve(derive_record_tokens(input))
}
fn derive_record_tokens(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::DeriveInput);
    match expand_record(&item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
fn expand_record(item: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let mut namespace: Option<LitStr> = None;
    let mut previous: Option<Type> = None;
    for attr in item
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("record"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("previous") {
                if previous.is_some() {
                    return Err(meta.error("previous record type may only be declared once"));
                }
                previous = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if !meta.path.is_ident("name") {
                return Err(meta.error("expected name = \"stable.record.name\""));
            }
            if namespace.is_some() {
                return Err(meta.error("record name may only be declared once"));
            }
            let name: LitStr = meta.value()?.parse()?;
            let value = name.value();
            if value.is_empty() || value.len() > 100 || value.chars().any(char::is_control) {
                return Err(syn::Error::new(
                    name.span(),
                    "record name must contain 1..=100 UTF8 bytes without control characters",
                ));
            }
            namespace = Some(name);
            Ok(())
        })?;
        if namespace.is_none() && previous.is_none() {
            return Err(syn::Error::new(
                attr.span(),
                "record attribute requires name = \"stable.record.name\"",
            ));
        }
    }
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new(
            item.generics.span(),
            "Record derive does not support generic records",
        ));
    }
    let syn::Data::Struct(data) = &item.data else {
        return Err(syn::Error::new(
            item.span(),
            "Record derive requires a named-field struct",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new(
            data.fields.span(),
            "Record derive requires named fields",
        ));
    };
    let mut hash = 0xcbf29ce484222325_u64;
    let mut encoders = Vec::new();
    let mut decoders = Vec::new();
    let mut projections = Vec::new();
    let mut nested_formats = Vec::new();
    let mut upgrades = Vec::new();
    for (index, field) in fields.named.iter().enumerate() {
        let mut field = field.clone();
        normalize_type(&mut field.ty);
        let field = &field;
        let mut default: Option<syn::Expr> = None;
        let mut nested = false;
        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("record"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("nested") {
                    if nested {
                        return Err(meta.error("nested may only be declared once"));
                    }
                    nested = true;
                    return Ok(());
                }
                if !meta.path.is_ident("default") {
                    return Err(meta.error(
                        "record name belongs on the struct, not a field; expected nested or default",
                    ));
                }
                if default.is_some() {
                    return Err(meta.error("record default may only be declared once"));
                }
                default = Some(if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse()?
                } else {
                    syn::parse_quote!(::core::default::Default::default())
                });
                Ok(())
            })?;
            if default.is_none() && !nested {
                return Err(syn::Error::new(
                    attr.span(),
                    "record field attribute requires default",
                ));
            }
        }
        if default.is_some() && previous.is_none() {
            return Err(syn::Error::new(
                field.span(),
                "record defaults require a previous record type",
            ));
        }
        let Type::Path(path) = &field.ty else {
            return Err(syn::Error::new(
                field.ty.span(),
                "Record fields must be scalar, Option or Vec of scalars, or marked #[record(nested)]",
            ));
        };
        let scalar = |ty: &Type| -> Option<String> {
            let Type::Path(path) = ty else {
                return None;
            };
            if path.qself.is_some() || path.path.segments.len() != 1 {
                return None;
            }
            let segment = &path.path.segments[0];
            let name = segment.ident.to_string();
            (matches!(name.as_str(), "String" | "i64" | "bool" | "u64" | "f64")
                && matches!(segment.arguments, PathArguments::None))
            .then_some(name)
        };
        let container_inner = |container: &str| -> Option<&Type> {
            if path.qself.is_some()
                || path.path.segments.len() != 1
                || path.path.segments[0].ident != container
            {
                return None;
            }
            let PathArguments::AngleBracketed(args) = &path.path.segments[0].arguments else {
                return None;
            };
            if args.args.len() != 1 {
                return None;
            }
            match args.args.first()? {
                GenericArgument::Type(inner) => Some(inner),
                _ => None,
            }
        };
        let scalar_list = (!nested)
            .then(|| container_inner("Vec").and_then(scalar))
            .flatten();
        let nested_optional = nested && container_inner("Option").is_some();
        let nested_list = nested && container_inner("Vec").is_some();
        let kind = if nested {
            Some(
                if nested_optional {
                    "Option<Record>"
                } else if nested_list {
                    "Vec<Record>"
                } else {
                    "Record"
                }
                .to_owned(),
            )
        } else {
            scalar(&field.ty)
                .or_else(|| {
                    container_inner("Option")
                        .and_then(scalar)
                        .map(|inner| format!("Option<{inner}>"))
                })
                .or_else(|| scalar_list.as_ref().map(|inner| format!("Vec<{inner}>")))
        };
        let Some(kind) = kind else {
            return Err(syn::Error::new(
                field.ty.span(),
                "Record fields must be scalar, Option or Vec of scalars, or marked #[record(nested)]",
            ));
        };
        let name = field.ident.as_ref().unwrap();
        let method = quote::format_ident!("{}_field", name.to_string().trim_start_matches("r#"));
        let field_type = &field.ty;
        let read = if nested {
            let ty = if nested_optional {
                container_inner("Option")
            } else if nested_list {
                container_inner("Vec")
            } else {
                None
            }
            .unwrap_or(&field.ty);
            if nested_optional {
                quote!(reader.optional_record::<#ty>())
            } else if nested_list {
                quote!(reader.records::<#ty>())
            } else {
                quote!(reader.record::<#ty>())
            }
        } else if scalar_list.is_some() {
            let ty = container_inner("Vec").unwrap();
            quote!(reader.fields::<#ty>())
        } else {
            quote!(reader.field::<#field_type>())
        };
        let visibility = &field.vis;
        projections.push(quote! {
            #[allow(dead_code)]
            #visibility fn #method() -> ::blockui::RecordField<Self, #field_type> {
                ::blockui::RecordField::__new(#index, ::blockui::abi::key(::core::stringify!(#name)), |field| {
                    let framed = ::std::format!("{}\n{}", field.len(), field);
                    let mut reader = ::blockui::record_codec::Reader::new(&framed)?;
                    let value = #read?;
                    reader.finish()?;
                    Ok(value)
                }, |record| &mut record.#name)
            }
        });
        upgrades.push(match default {
            Some(value) => quote!(#name: #value),
            None => quote!(#name: previous.#name),
        });
        for byte in format!("{name}:{kind};").bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
        if nested {
            let ty = if nested_optional {
                container_inner("Option")
            } else if nested_list {
                container_inner("Vec")
            } else {
                None
            }
            .unwrap_or(&field.ty);
            nested_formats.push(quote!(<#ty as ::blockui::Record>::FORMAT));
            if nested_optional {
                encoders.push(quote!(::blockui::record_codec::append_optional_record(&mut output, &self.#name)?;));
                decoders.push(quote!(#name: reader.optional_record::<#ty>()?));
            } else if nested_list {
                encoders.push(
                    quote!(::blockui::record_codec::append_records(&mut output, &self.#name)?;),
                );
                decoders.push(quote!(#name: reader.records::<#ty>()?));
            } else {
                encoders.push(
                    quote!(::blockui::record_codec::append_record(&mut output, &self.#name)?;),
                );
                decoders.push(quote!(#name: reader.record::<#ty>()?));
            }
        } else if scalar_list.is_some() {
            let ty = container_inner("Vec").unwrap();
            encoders
                .push(quote!(::blockui::record_codec::append_fields(&mut output, &self.#name)?;));
            decoders.push(quote!(#name: reader.fields::<#ty>()?));
        } else {
            encoders.push(quote!(::blockui::record_codec::append(&mut output, &self.#name)?;));
            decoders.push(quote!(#name: reader.field()?));
        }
    }
    let name = &item.ident;
    let fingerprint = format!("@record1-{hash:016x}");
    let mut format = match namespace.as_ref() {
        Some(namespace) => quote!(::core::concat!(#namespace, #fingerprint)),
        None => quote!(
            ::core::concat!(::core::module_path!(), "::", ::core::stringify!(#name), #fingerprint)
        ),
    };
    if !nested_formats.is_empty() {
        let prefix = match namespace.as_ref() {
            Some(namespace) => quote!(::core::concat!(#namespace, "@record1-")),
            None => quote!(::core::concat!(
                ::core::module_path!(),
                "::",
                ::core::stringify!(#name),
                "@record1-"
            )),
        };
        format = quote!({
            const PREFIX: &str = #prefix;
            const HASH: [u8; 16] = ::blockui::record_codec::nested_fingerprint(#hash, &[#(#nested_formats),*]);
            const NAME: [u8; PREFIX.len() + 16] = ::blockui::record_codec::schema_name::<{PREFIX.len() + 16}>(PREFIX, HASH);
            match ::core::str::from_utf8(&NAME) {
                Ok(name) => name,
                Err(_) => panic!("Invalid record schema name"),
            }
        });
    }
    let upgrade_encoded = previous.as_ref().map(|previous| {
        quote! {
            const PREVIOUS_FORMAT: ::core::option::Option<&'static str> = Some(<#previous as ::blockui::Record>::FORMAT);
            fn upgrade_encoded(encoded: &str) -> ::blockui::Result<Self> {
                let previous = ::blockui::record_codec::decode_record::<#previous>(encoded)?;
                <Self as ::blockui::RecordUpgrade<#previous>>::upgrade(previous)
            }
        }
    });
    let upgrade = previous.map(|previous| {
        quote! {
            impl ::blockui::RecordUpgrade<#previous> for #name {
                fn upgrade(previous: #previous) -> ::blockui::Result<Self> {
                    let _ = &previous;
                    Ok(Self { #( #upgrades, )* })
                }
            }
        }
    });
    Ok(quote! {
        #upgrade
        impl #name { #(#projections)* }
        impl ::blockui::Record for #name {
            const FORMAT: &'static str = #format;
            #upgrade_encoded
            fn encode(&self) -> ::blockui::Result<::std::string::String> {
                let mut output = ::std::string::String::new();
                #(#encoders)*
                Ok(output)
            }
            fn decode(payload: &str) -> ::blockui::Result<Self> {
                let mut reader = ::blockui::record_codec::Reader::new(payload)?;
                let value = Self { #(#decoders,)* };
                reader.finish()?;
                Ok(value)
            }
        }
    })
}
// Only propagate conditional presence, not field/method-only attributes such as
// doc or inline, onto generated schema expressions and registration entries.
fn conditional_attributes(attrs: &[syn::Attribute]) -> syn::Result<Vec<syn::Attribute>> {
    fn presence(meta: &syn::Meta) -> syn::Result<Option<syn::Meta>> {
        if meta.path().is_ident("cfg") {
            return Ok(Some(meta.clone()));
        }
        if !meta.path().is_ident("cfg_attr") {
            return Ok(None);
        }
        let syn::Meta::List(list) = meta else {
            return Err(syn::Error::new(
                meta.span(),
                "expected cfg_attr condition and attributes",
            ));
        };
        let args = list.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        )?;
        let mut args = args.iter();
        let condition = args
            .next()
            .ok_or_else(|| syn::Error::new(list.span(), "missing cfg_attr condition"))?;
        let nested = args.map(presence).collect::<syn::Result<Vec<_>>>()?;
        let nested: Vec<_> = nested.into_iter().flatten().collect();
        Ok((!nested.is_empty()).then(|| syn::parse_quote!(cfg_attr(#condition, #(#nested),*))))
    }
    attrs
        .iter()
        .filter_map(|attr| match presence(&attr.meta) {
            Ok(Some(meta)) => Some(Ok(syn::parse_quote!(#[#meta]))),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}
#[proc_macro_attribute]
pub fn actor(attributes: TokenStream, input: TokenStream) -> TokenStream {
    sdk_path::resolve(actor_tokens(attributes, input))
}
fn actor_tokens(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemStruct);
    let namespace = match actor_namespace(attributes.into(), item.ident.to_string()) {
        Ok(namespace) => namespace,
        Err(error) => return error.into_compile_error().into(),
    };
    match expand(&mut item, &namespace) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
/// An application root with automatically scoped component instances.
#[proc_macro_attribute]
pub fn app(attributes: TokenStream, input: TokenStream) -> TokenStream {
    sdk_path::resolve(app_tokens(attributes, input))
}
fn app_tokens(attributes: TokenStream, input: TokenStream) -> TokenStream {
    if let Ok(function) = opaque_impl::parse_function(input.clone().into()) {
        return match function_component::expand(function, attributes.into(), true) {
            Ok(tokens) => tokens.into(),
            Err(error) => error.into_compile_error().into(),
        };
    }
    let mut item = parse_macro_input!(input as ItemStruct);
    let namespace = match actor_namespace(attributes.into(), item.ident.to_string()) {
        Ok(value) => value,
        Err(error) => return error.into_compile_error().into(),
    };
    match expand_declaration(&mut item, &namespace, true) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
/// Ordinary application methods and a turn-local view.
#[proc_macro_attribute]
pub fn app_impl(attributes: TokenStream, input: TokenStream) -> TokenStream {
    sdk_path::resolve(app_impl_tokens(attributes, input))
}
fn app_impl_tokens(attributes: TokenStream, input: TokenStream) -> TokenStream {
    if !attributes.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "app_impl accepts no arguments",
        )
        .into_compile_error()
        .into();
    }
    let mut item = match opaque_impl::parse(input.into()) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error().into(),
    };
    infer_bound_methods(&mut item);
    match expand_implementation_kind(&mut item, true, true, true) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
fn infer_bound_methods(item: &mut syn::ItemImpl) {
    for member in &mut item.items {
        let syn::ImplItem::Fn(method) = member else {
            continue;
        };
        // A top-level UI marker opts a receiver UI helper into block sugar.
        // Nested items and other macro inputs own their syntax; ui! is an explicit marker.
        if method.sig.receiver().is_some()
            && !method.attrs.iter().any(|attr| {
                matches!(
                    attr.path()
                        .segments
                        .last()
                        .map(|segment| segment.ident.to_string())
                        .as_deref(),
                    Some("action" | "computed" | "scope")
                )
            })
            && !method.attrs.iter().any(|attr| {
                attr.path()
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "view")
            })
            && view_blocks::has_ui_marker({
                let body = &method.block;
                quote!(#body)
            })
        {
            method
                .attrs
                .push(syn::parse_quote!(#[::blockui::view(block)]));
        }
        let has_context = method.sig.inputs.iter().skip(1).any(|input| {
            let syn::FnArg::Typed(input) = input else { return false; };
            let mut ty = input.ty.as_ref();
            loop {
                ty = match ty {
                    Type::Reference(reference) => &reference.elem,
                    Type::Group(group) => &group.elem,
                    Type::Paren(paren) => &paren.elem,
                    _ => break,
                };
            }
            matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|s| s.ident == "Cx"))
        });
        if has_context {
            continue;
        }
        for attr in &mut method.attrs {
            if !["action", "computed", "scope"]
                .iter()
                .any(|name| attr.path().is_ident(name))
            {
                continue;
            }
            let path = attr.path().clone();
            match &attr.meta {
                syn::Meta::Path(_) => *attr = syn::parse_quote!(#[#path(bound)]),
                syn::Meta::List(list)
                    if !list.parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
                    ).is_ok_and(|arguments| arguments.iter().any(|argument|
                        matches!(argument, syn::Meta::Path(path) if path.is_ident("bound")))) =>
                {
                    let tokens = &list.tokens;
                    *attr = syn::parse_quote!(#[#path(bound, #tokens)]);
                }
                _ => {}
            }
        }
    }
}
/// Declare component-local runtime state without creating an actor lifecycle.
#[proc_macro_attribute]
pub fn component(attributes: TokenStream, input: TokenStream) -> TokenStream {
    sdk_path::resolve(component_tokens(attributes, input))
}
fn component_tokens(attributes: TokenStream, input: TokenStream) -> TokenStream {
    if let Ok(function) = opaque_impl::parse_function(input.clone().into()) {
        return match function_component::expand(function, attributes.into(), false) {
            Ok(tokens) => tokens.into(),
            Err(error) => error.into_compile_error().into(),
        };
    }
    let mut item = parse_macro_input!(input as ItemStruct);
    let namespace = match actor_namespace(attributes.into(), item.ident.to_string()) {
        Ok(value) => value,
        Err(error) => return error.into_compile_error().into(),
    };
    match expand_declaration(&mut item, &namespace, true) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

/// Declare a component's actions and bound renderer; the parent registers them.
#[proc_macro_attribute]
pub fn component_impl(attributes: TokenStream, input: TokenStream) -> TokenStream {
    sdk_path::resolve(component_impl_tokens(attributes, input))
}
fn component_impl_tokens(attributes: TokenStream, input: TokenStream) -> TokenStream {
    if !attributes.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "component_impl accepts no arguments",
        )
        .into_compile_error()
        .into();
    }
    let mut item = match opaque_impl::parse(input.into()) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error().into(),
    };
    infer_bound_methods(&mut item);
    match expand_implementation_mode(&mut item, true, true) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn actor_namespace(attributes: proc_macro2::TokenStream, default: String) -> syn::Result<String> {
    use syn::parse::Parser;
    let mut namespace = default;
    let mut seen = false;
    syn::meta::parser(|meta| {
        if !meta.path.is_ident("name") {
            return Err(meta.error("expected name = \"stable.actor.name\""));
        }
        if seen {
            return Err(meta.error("actor name may only be declared once"));
        }
        seen = true;
        namespace = meta.value()?.parse::<LitStr>()?.value();
        Ok(())
    })
    .parse2(attributes)?;
    Ok(namespace)
}
fn host_identity(
    field: &mut syn::Field,
    attribute_name: &str,
    namespace: &str,
) -> syn::Result<(LitStr, syn::Expr)> {
    let name = field.ident.as_ref().unwrap();
    let mut key = None;
    let mut policy = None;
    for attribute in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident(attribute_name))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") && key.is_none() {
                key = Some(meta.value()?.parse::<LitStr>()?);
                Ok(())
            } else if meta.path.is_ident("reload") && policy.is_none() {
                policy = Some(meta.value()?.parse::<syn::Expr>()?);
                Ok(())
            } else {
                Err(meta.error("expected unique key = \"stable.name\" or reload = Rust expression"))
            }
        })?;
    }
    let key = key.unwrap_or_else(|| LitStr::new(&format!("{namespace}.{name}"), name.span()));
    if key.value().is_empty() {
        return Err(syn::Error::new(
            key.span(),
            "host identity key cannot be empty",
        ));
    }
    field
        .attrs
        .retain(|attribute| !attribute.path().is_ident(attribute_name));
    Ok((
        key,
        policy.unwrap_or_else(|| syn::parse_quote!(::blockui::ReloadPolicy::Preserve)),
    ))
}

fn expand(item: &mut ItemStruct, namespace: &str) -> syn::Result<proc_macro2::TokenStream> {
    expand_declaration(item, namespace, false)
}
fn expand_declaration(
    item: &mut ItemStruct,
    namespace: &str,
    instance_scopes: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    if namespace.is_empty() {
        return Err(syn::Error::new(
            item.ident.span(),
            "actor name cannot be empty",
        ));
    }
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new(
            item.generics.span(),
            "actor declarations do not yet support generics",
        ));
    }
    let Fields::Named(fields) = &mut item.fields else {
        return Err(syn::Error::new(
            item.fields.span(),
            "actor requires named State fields",
        ));
    };
    let mut unconditional_keys = std::collections::BTreeMap::new();
    let mut active_task_keys = Vec::new();
    let mut active_resource_keys = Vec::new();
    let mut task_keys = std::collections::BTreeSet::new();
    let mut resource_keys = std::collections::BTreeSet::new();
    let mut initializers = Vec::new();
    let mut specs = Vec::new();
    let mut codec_checks = Vec::new();
    let mut route_checks = Vec::new();
    let mut record_initializers = Vec::new();
    let mut scoped_fields = Vec::new();
    let mut bound_fields = Vec::new();
    let mut bound_initializers = Vec::new();
    let mut component_types = Vec::new();
    for field in &mut fields.named {
        normalize_type(&mut field.ty);
        // An unannotated named field composes a stateful component. Explicit
        // record declarations retain their existing state-only semantics.
        let inferred_component = !field.attrs.iter().any(|a| {
            a.path().is_ident("state") || a.path().is_ident("task") || a.path().is_ident("resource")
        }) && matches!(&field.ty, Type::Path(path) if path.qself.is_none()
                && path.path.segments.last().is_some_and(|last| matches!(last.arguments, PathArguments::None)
                    && !matches!(last.ident.to_string().as_str(), "Task" | "State" | "String" | "bool" | "i64" | "f64" | "i32" | "u64" | "usize")));
        if inferred_component {
            let ty = field.ty.clone();
            let cfg = conditional_attributes(&field.attrs)?;
            let scope = instance_scopes
                .then(|| LitStr::new(&field.ident.as_ref().unwrap().to_string(), field.span()));
            component_types.push((cfg, ty, scope.clone()));
            field.attrs.push(match scope {
                Some(scope) => syn::parse_quote!(#[state(record, scope = #scope)]),
                None => syn::parse_quote!(#[state(record)]),
            });
        }
        let name = field.ident.as_ref().unwrap();
        let field_visibility = &field.vis;
        let field_type = field.ty.clone();
        let cfg = conditional_attributes(&field.attrs)?;
        let task_field = matches!(&field.ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Task" && matches!(segment.arguments, PathArguments::None)));
        let file_resource = matches!(&field.ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| {
            segment.ident == "Resource" && matches!(&segment.arguments, PathArguments::AngleBracketed(arguments)
                if arguments.args.len() == 1 && matches!(arguments.args.first(), Some(GenericArgument::Type(Type::Path(marker)))
                    if marker.path.segments.last().is_some_and(|last| (last.ident == "File" || last.ident == "Socket") && matches!(last.arguments, PathArguments::None))))
        }));
        for attribute in &field.attrs {
            if attribute.path().is_ident("task") && !task_field {
                return Err(syn::Error::new(
                    attribute.span(),
                    "task attributes require a Task field",
                ));
            }
            if attribute.path().is_ident("resource") && !file_resource {
                return Err(syn::Error::new(
                    attribute.span(),
                    "resource attributes require a Resource<File> or Resource<Socket> field",
                ));
            }
        }
        bound_fields.push(quote!(#(#cfg)* #field_visibility #name: <#field_type as ::blockui::BindState>::Bound<'cx, 'host>));
        let binding_field = format!("{namespace}.{name}");
        bound_initializers.push(
            quote!(#(#cfg)* #name: match ::blockui::BindState::bind(&self.#name, cx) {
                Ok(value) => value,
                Err(error) => return Err(error.__binding_field(#binding_field)),
            }),
        );
        if task_field {
            if field
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("state"))
            {
                return Err(syn::Error::new(
                    field.span(),
                    "Task fields are host task identities, not durable state slots",
                ));
            }
            let (key, policy) = host_identity(field, "task", namespace)?;
            active_task_keys.push(quote!(#(#cfg)* ::blockui::abi::key(#key)));
            if cfg.is_empty() && !task_keys.insert(key.value()) {
                return Err(syn::Error::new(
                    key.span(),
                    "duplicate task key; each unconditional field needs a distinct key",
                ));
            }
            let name = field.ident.as_ref().unwrap();
            initializers
                .push(quote!(#(#cfg)* #name: ::blockui::Task::new(#key).on_reload(#policy)));
            scoped_fields.push(quote!(#(#cfg)* #name: self.#name.scoped(scope)));
            continue;
        }
        if file_resource {
            if field
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("state"))
            {
                return Err(syn::Error::new(
                    field.span(),
                    "Resource fields are host resource identities, not durable state slots",
                ));
            }
            let (key, policy) = host_identity(field, "resource", namespace)?;
            active_resource_keys.push(quote!(#(#cfg)* ::blockui::abi::key(#key)));
            if cfg.is_empty() && !resource_keys.insert(key.value()) {
                return Err(syn::Error::new(
                    key.span(),
                    "duplicate resource key; each unconditional field needs a distinct key",
                ));
            }
            let name = field.ident.as_ref().unwrap();
            initializers.push(quote!(#(#cfg)* #name: <#field_type>::new(#key).on_reload(#policy)));
            scoped_fields.push(quote!(#(#cfg)* #name: self.#name.scoped(scope)));
            continue;
        }
        let record = field.attrs.iter().any(|attribute| {
            attribute.path().is_ident("state")
                && attribute
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok_and(|metas| {
                        metas.iter().any(
                            |meta| matches!(meta, syn::Meta::Path(path) if path.is_ident("record")),
                        )
                    })
        });
        if record {
            let mut scope: Option<LitStr> = None;
            let mut seen = false;
            for attribute in field
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("state"))
            {
                attribute.parse_nested_meta(|meta| {
                    if meta.path.is_ident("record") && !seen {
                        seen = true;
                        Ok(())
                    } else if meta.path.is_ident("scope") && scope.is_none() {
                        scope = Some(meta.value()?.parse()?);
                        Ok(())
                    } else {
                        Err(meta.error(
                            "expected record and an optional unique scope = \"stable.name\"",
                        ))
                    }
                })?;
            }
            field
                .attrs
                .retain(|attribute| !attribute.path().is_ident("state"));
            let conditional = conditional_attributes(&field.attrs)?;
            let ty = &field.ty;
            if let Some(scope) = scope {
                if scope.value().is_empty() {
                    return Err(syn::Error::new(
                        scope.span(),
                        "record scope cannot be empty",
                    ));
                }
                initializers.push(quote!(#(#conditional)* #name: <#ty>::new().scoped(::blockui::Key::named(#scope))));
                specs.push(quote!(#(#conditional)* &::blockui::__scope_schema::<{<#ty>::SCHEMA.len()}>(<#ty>::SCHEMA, ::blockui::Key::named(#scope))));
                codec_checks.push(quote!(#(#conditional)* {
                    let mut nested_scopes = vec![::blockui::Key::named(#scope)];
                    nested_scopes.extend_from_slice(scopes);
                    <#ty>::__validate_record_codecs_with_scopes(cx, previous, &nested_scopes)?;
                }));
                route_checks.push(quote!(#(#conditional)* {
                    let mut nested_scopes = vec![::blockui::Key::named(#scope)];
                    nested_scopes.extend_from_slice(scopes);
                    <#ty>::__validate_route_histories_with_scopes(cx, &nested_scopes)?;
                }));
                record_initializers.push(quote!(#(#conditional)* {
                    let mut nested_scopes = vec![::blockui::Key::named(#scope)];
                    nested_scopes.extend_from_slice(scopes);
                    <#ty>::__initialize_record_values_with_scopes(cx, &nested_scopes)?;
                }));
            } else {
                initializers.push(quote!(#(#conditional)* #name: <#ty>::new()));
                specs.push(quote!(#(#conditional)* <#ty>::SCHEMA));
                codec_checks.push(quote!(#(#conditional)* {
                    <#ty>::__validate_record_codecs_with_scopes(cx, previous, scopes)?;
                }));
                route_checks.push(quote!(#(#conditional)* {
                    <#ty>::__validate_route_histories_with_scopes(cx, scopes)?;
                }));
                record_initializers.push(quote!(#(#conditional)* {
                    <#ty>::__initialize_record_values_with_scopes(cx, scopes)?;
                }));
            }
            scoped_fields.push(quote!(#(#conditional)* #name: self.#name.scoped(scope)));
            continue;
        }

        let Type::Path(path) = &field.ty else {
            return Err(syn::Error::new(
                field.ty.span(),
                "expected State<i64>, State<f64>, State<bool> or State<String>",
            ));
        };
        let segment = path.path.segments.last().unwrap();
        let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return Err(syn::Error::new(
                field.ty.span(),
                "expected State<i64>, State<f64>, State<bool> or State<String>",
            ));
        };
        let Some(GenericArgument::Type(Type::Path(value))) = arguments.args.first() else {
            return Err(syn::Error::new(
                arguments.span(),
                "expected a State value type",
            ));
        };
        let kind = value.path.segments.last().unwrap().ident.to_string();
        let list = kind == "Vec"
            && matches!(&value.path.segments.last().unwrap().arguments,
            PathArguments::AngleBracketed(args) if args.args.len() == 1 && matches!(args.args.first(), Some(GenericArgument::Type(Type::Path(path))) if path.path.is_ident("i64") || {
                let names: Vec<_> = path.path.segments.iter().map(|segment| segment.ident.to_string()).collect();
                path.qself.is_none()
                    && path.path.segments.iter().all(|segment| matches!(segment.arguments, PathArguments::None))
                    && matches!(names.as_slice(), [root, primitive, name] if (root == "core" || root == "std") && primitive == "primitive" && name == "i64")
            }));

        let text_list = kind == "Vec"
            && matches!(&value.path.segments.last().unwrap().arguments,
                PathArguments::AngleBracketed(args) if args.args.len() == 1
                && matches!(args.args.first(), Some(GenericArgument::Type(Type::Path(path)))
                    if path.qself.is_none() && path.path.segments.last().is_some_and(|segment| segment.ident == "String" && matches!(segment.arguments, PathArguments::None))));

        let record_value = !(matches!(kind.as_str(), "i64" | "f64" | "bool" | "String") || list || text_list)
            || field.attrs.iter().filter(|attr| attr.path().is_ident("state"))
            .any(|attr| attr.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                .is_ok_and(|metas| metas.iter().any(|meta| matches!(meta, syn::Meta::Path(path) if path.is_ident("record_value")))));
        if segment.ident != "State"
            || arguments.args.len() != 1
            || !(matches!(kind.as_str(), "i64" | "f64" | "bool" | "String")
                || list
                || text_list
                || record_value)
        {
            return Err(syn::Error::new(
                field.ty.span(),
                "expected State<T>; T must be a supported scalar/list or implement Record",
            ));
        }
        let value_type = &value;
        let mut key: syn::Expr = {
            let literal = LitStr::new(&format!("{namespace}.{name}"), name.span());
            syn::parse_quote!(#literal)
        };
        let mut default: syn::Expr = if record_value {
            syn::parse_quote!(<#value_type as ::core::default::Default>::default())
        } else if kind == "i64" {
            syn::parse_quote!(0)
        } else if kind == "f64" {
            syn::parse_quote!(0.0)
        } else if kind == "bool" {
            syn::parse_quote!(false)
        } else if list {
            syn::parse_quote!(&[])
        } else {
            syn::parse_quote!("")
        };
        let mut key_seen = false;
        let mut default_seen = false;
        let mut codec: Option<Type> = None;
        let mut route: Option<Type> = None;
        let mut route_request: Option<Type> = None;
        let mut record_value_seen = false;
        for attribute in field
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("state"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("record_value") && !record_value_seen {
                    record_value_seen = true;
                    Ok(())
                } else if meta.path.is_ident("key") && !key_seen {
                    key_seen = true;
                    key = meta.value()?.parse()?;
                    Ok(())
                } else if meta.path.is_ident("default") && !default_seen {
                    default_seen = true;
                    default = meta.value()?.parse()?;
                    Ok(())
                } else if meta.path.is_ident("codec")
                    && codec.is_none()
                    && route.is_none()
                    && route_request.is_none()
                {
                    codec = Some(meta.value()?.parse()?);
                    Ok(())
                } else if meta.path.is_ident("route")
                    && route.is_none()
                    && codec.is_none()
                    && route_request.is_none()
                {
                    route = Some(meta.value()?.parse()?);
                    Ok(())
                } else if meta.path.is_ident("route_request")
                    && route_request.is_none()
                    && route.is_none()
                    && codec.is_none()
                {
                    route_request = Some(meta.value()?.parse()?);
                    Ok(())
                } else {
                    Err(meta.error("expected a unique key, default or codec/route"))
                }
            })?;
        }
        if codec.is_some() && !text_list {
            return Err(syn::Error::new(
                field.ty.span(),
                "codec requires State<Vec<String>>",
            ));
        }
        if route.is_some() && !text_list {
            return Err(syn::Error::new(
                field.ty.span(),
                "route requires State<Vec<String>>",
            ));
        }
        if route_request.is_some() && kind != "String" {
            return Err(syn::Error::new(
                field.ty.span(),
                "route_request requires State<String>",
            ));
        }
        if record_value && (codec.is_some() || route.is_some() || route_request.is_some()) {
            return Err(syn::Error::new(
                field.ty.span(),
                "record_value cannot combine with codec or route annotations",
            ));
        }
        let literal_key = match &key {
            syn::Expr::Lit(literal) => match &literal.lit {
                syn::Lit::Str(value) => Some(value.value()),
                _ => None,
            },
            _ => None,
        };
        if literal_key.as_deref() == Some("") {
            return Err(syn::Error::new(key.span(), "state key cannot be empty"));
        }
        field
            .attrs
            .retain(|attribute| !attribute.path().is_ident("state"));
        let conditional = conditional_attributes(&field.attrs)?;
        if conditional.is_empty()
            && let Some(value) = literal_key
            && let Some(previous) = unconditional_keys.insert(value.clone(), key.span())
        {
            let mut error = syn::Error::new(
                key.span(),
                format!(
                    "duplicate durable state key `{value}`; each unconditional field needs a distinct key"
                ),
            );
            error.combine(syn::Error::new(
                previous,
                "first field with this state key is here",
            ));
            return Err(error);
        }
        let constructor = if record_value {
            quote!(::blockui::State::<#value_type>::new(#key))
        } else if text_list && default_seen {
            quote!(::blockui::State::<#value_type>::__with_defaults(
                #key,
                const { &::blockui::__text_list_defaults::<{ (#default as &[&str]).len() }>(#default) }
            ))
        } else if text_list {
            quote!(::blockui::State::<#value_type>::new(#key))
        } else if kind == "i64" || kind == "f64" || kind == "bool" {
            quote!(::blockui::State::<#value_type>::new(#key, #default))
        } else {
            quote!(::blockui::State::<#value_type>::with_default(#key, #default))
        };
        initializers.push(quote!(#(#conditional)* #name: #constructor));
        scoped_fields.push(quote!(#(#conditional)* #name: self.#name.scoped(scope)));
        // Evaluate only this field when assembling the constant schema.
        specs.push(quote!(#(#conditional)* &[(#constructor).spec()]));
        if let Some(codec) = &codec {
            route_checks.push(quote!(#(#conditional)* {
                let mut collection = Self::new().#name;
                for scope in scopes { collection = collection.scoped(*scope); }
                ::blockui::RecordMap::<#codec>::new(collection).entries(cx)?;
            }));
        }
        let format = if record_value {
            record_initializers.push(quote!(#(#conditional)* {
                let mut value = Self::new().#name;
                for scope in scopes { value = value.scoped(*scope); }
                value.__initialize(cx, || #default)?;
            }));
            route_checks.push(quote!(#(#conditional)* {
                let mut value = Self::new().#name;
                for scope in scopes { value = value.scoped(*scope); }
                value.get(cx)?;
            }));
            Some(quote!(<#value_type as ::blockui::Record>::FORMAT))
        } else {
            codec
                .as_ref()
                .map(|codec| quote!(<#codec as ::blockui::Record>::FORMAT))
        };
        let route_format = route
            .as_ref()
            .or(route_request.as_ref())
            .map(|route| quote!(<#route as ::blockui::RoutePath>::FORMAT));
        let previous_probe = if route_request.is_some() {
            quote!(collection.previous(cx))
        } else {
            quote!(collection.__previous_len(cx))
        };
        if let Some(request) = route_request {
            route_checks.push(quote!(#(#conditional)* {
                let mut request = Self::new().#name;
                for scope in scopes { request = request.scoped(*scope); }
                ::blockui::__validate_pending_route::<#request>(&request.get(cx)?)?;
            }));
        }
        if let Some(route) = route {
            route_checks.push(quote!(#(#conditional)* {
                let mut collection = Self::new().#name;
                for scope in scopes { collection = collection.scoped(*scope); }
                ::blockui::PathRouter::<#route>::new(collection).current(cx)?;
            }));
        }
        let error = if route_format.is_some() {
            "Route format changed; explicit migration is required"
        } else {
            "Record codec changed; explicit migration is required"
        };
        if let Some(format) = format.or(route_format) {
            let changed_format = if record_value {
                quote!(collection.__upgrade_previous(cx)?;)
            } else if let Some(codec) = &codec {
                quote! {
                    let previous_format = <#codec as ::blockui::Record>::PREVIOUS_FORMAT
                        .ok_or_else(|| ::blockui::Error::Application(#error.into()))?;
                    let mut previous_marker = ::blockui::State::<i64>::new(previous_format, 0)
                        .scoped(::blockui::Key::named(#key));
                    for scope in scopes { previous_marker = previous_marker.scoped(*scope); }
                    previous_marker.previous(cx).map_err(|_| ::blockui::Error::Application(
                        "Record predecessor format requires migration".into()
                    ))?;
                    ::blockui::RecordMap::<#codec>::new(collection).__upgrade_previous(cx)?;
                }
            } else {
                quote!(return Err(::blockui::Error::Application(#error.into()));)
            };
            // Scope the marker by collection identity so two collections can
            // use one codec without generating duplicate state keys.
            specs.push(quote!(#(#conditional)* &[
                ::blockui::State::<i64>::new(const {
                    assert!(!#format.is_empty(), "state codec/route requires a nonempty FORMAT");
                    #format
                }, 0)
                    .scoped(::blockui::Key::named(#key)).spec()
            ]));
            codec_checks.push(quote!(#(#conditional)* {
                let mut collection = Self::new().#name;
                let mut marker = ::blockui::State::<i64>::new(#format, 0)
                    .scoped(::blockui::Key::named(#key));
                for scope in scopes { collection = collection.scoped(*scope); marker = marker.scoped(*scope); }
                if #previous_probe.is_ok() {
                    if marker.previous(cx).is_err() {
                        #changed_format
                    }
                }
            }));
        }
    }
    fields
        .named
        .push(syn::parse_quote!(#[doc(hidden)] __actor_identity: ::blockui::InstancePath));
    initializers.push(quote!(__actor_identity: ::blockui::InstancePath::EMPTY));
    scoped_fields.push(quote!(__actor_identity: self.__actor_identity.scoped(scope)));
    bound_initializers.push(quote!(__actor_identity: self.__actor_identity));
    let has_fields = fields.named.len() > 1;
    if has_fields {
        for field in &mut fields.named {
            if field
                .ident
                .as_ref()
                .is_some_and(|id| id == "__actor_identity")
            {
                continue;
            }
            let ty = &field.ty;
            field.ty = syn::parse_quote!(<__StateMode as ::blockui::StateMode>::Field<#ty>);
        }
        fields.named.push(syn::parse_quote!(
            __actor_context: <__StateMode as ::blockui::StateMode>::Context
        ));
        initializers.push(quote!(__actor_context: ()));
        scoped_fields.push(quote!(__actor_context: ()));
        bound_initializers.push(quote!(__actor_context: cx));
        item.generics
            .params
            .push(syn::parse_quote!(__StateMode: ::blockui::StateMode = ::blockui::DescriptorMode));
    }
    let name = &item.ident;
    let bound_name = quote::format_ident!("{}Bound", name);
    let visibility = &item.vis;
    let bound_declaration = if has_fields {
        quote!(#[allow(non_camel_case_types)] #visibility type #bound_name<'cx, 'host> = #name<::blockui::TurnMode<'cx, 'host>>;)
    } else {
        quote!(#[allow(non_camel_case_types)] #visibility struct #bound_name<'cx, 'host: 'cx> {
            #(#bound_fields,)*
            __actor_identity: ::blockui::InstancePath,
            __actor_context: &'cx ::blockui::Cx<'host>,
        })
    };
    let bound_value = if has_fields {
        quote!(#name { #(#bound_initializers,)* })
    } else {
        quote!(#bound_name { #(#bound_initializers,)* __actor_context: cx })
    };
    let component_actions = component_types.iter().map(|(cfg, ty, scope)| match scope {
        Some(scope) => quote!(#(#cfg)* &::blockui::__scope_actions::<{<#ty as ::blockui::Component>::ACTIONS.len()}>(<#ty as ::blockui::Component>::ACTIONS, ::blockui::Key::named(#scope))),
        None => quote!(#(#cfg)* <#ty as ::blockui::Component>::ACTIONS),
    });
    let component_scopes = component_types.iter().map(|(cfg, ty, scope)| match scope {
        Some(scope) => quote!(#(#cfg)* &::blockui::__scope_views::<{<#ty as ::blockui::Component>::SCOPES.len()}>(<#ty as ::blockui::Component>::SCOPES, ::blockui::Key::named(#scope))),
        None => quote!(#(#cfg)* <#ty as ::blockui::Component>::SCOPES),
    });
    let component_computeds = component_types.iter().map(|(cfg, ty, scope)| match scope {
        Some(scope) => quote!(#(#cfg)* &::blockui::__scope_computeds::<{<#ty as ::blockui::Component>::COMPUTEDS.len()}>(<#ty as ::blockui::Component>::COMPUTEDS, ::blockui::Key::named(#scope))),
        None => quote!(#(#cfg)* <#ty as ::blockui::Component>::COMPUTEDS),
    });
    let component_migrations = component_types.iter().map(|(cfg, ty, scope)| match scope {
        Some(scope) => quote!(#(#cfg)* <#ty as ::blockui::Component>::migrate(&cx.__child(::blockui::Key::named(#scope)), previous)?;),
        None => quote!(#(#cfg)* <#ty as ::blockui::Component>::migrate(cx, previous)?;),
    });
    let component_migration = if component_types.is_empty() {
        quote! {
            #[doc(hidden)] #[allow(non_upper_case_globals)]
            pub const __migrate_components: fn(&::blockui::Cx<'_>, u64) -> ::blockui::Result<()> = ::blockui::__empty_migration;
        }
    } else {
        quote! {
            #[doc(hidden)]
            pub fn __migrate_components(cx: &::blockui::Cx<'_>, previous: u64) -> ::blockui::Result<()> {
                let _ = (cx, previous);
                #(#component_migrations)*
                Ok(())
            }
        }
    };
    let record_initialization = if record_initializers.is_empty() {
        quote! {
            #[doc(hidden)] #[allow(non_upper_case_globals)]
            pub const __initialize_record_values_with_scopes: fn(&::blockui::Cx<'_>, &[::blockui::Key]) -> ::blockui::Result<()> = ::blockui::__empty_scoped_validation;
        }
    } else {
        quote! {
            #[doc(hidden)]
            pub fn __initialize_record_values_with_scopes(cx: &::blockui::Cx<'_>, scopes: &[::blockui::Key]) -> ::blockui::Result<()> {
                let _ = (cx, scopes);
                #(#record_initializers)*
                Ok(())
            }
        }
    };
    let route_validation = if route_checks.is_empty() {
        quote! {
            #[doc(hidden)] #[allow(non_upper_case_globals)]
            pub const __validate_route_histories_with_scopes: fn(&::blockui::Cx<'_>, &[::blockui::Key]) -> ::blockui::Result<()> = ::blockui::__empty_scoped_validation;
        }
    } else {
        quote! {
            #[doc(hidden)]
            pub fn __validate_route_histories_with_scopes(cx: &::blockui::Cx<'_>, scopes: &[::blockui::Key]) -> ::blockui::Result<()> {
                let _ = (cx, scopes);
                #(#route_checks)*
                Ok(())
            }
        }
    };
    let codec_validation = if codec_checks.is_empty() {
        quote! {
            #[doc(hidden)] #[allow(non_upper_case_globals)]
            pub const __validate_record_codecs_with_scopes: fn(&::blockui::Cx<'_>, u64, &[::blockui::Key]) -> ::blockui::Result<()> = ::blockui::__empty_record_validation;
        }
    } else {
        quote! {
            #[doc(hidden)]
            pub fn __validate_record_codecs_with_scopes(cx: &::blockui::Cx<'_>, previous: u64, scopes: &[::blockui::Key]) -> ::blockui::Result<()> {
                let _ = (cx, scopes);
                if previous != 0 { #(#codec_checks)* }
                Ok(())
            }
        }
    };
    Ok(quote! {
        #item
        const _: () = {
            ::blockui::__validate_host_keys(&[#(#active_task_keys),*]);
            ::blockui::__validate_host_keys(&[#(#active_resource_keys),*]);
        };
        #bound_declaration
        impl<'cx, 'host: 'cx> #bound_name<'cx, 'host> {
            /// Construct a typed, lazy property computation with a fresh receiver.
            /// Text and style properties mount it in their own dynamic observer.
            /// Accepts noncapturing closures or functions; no view context is retained.
            pub fn live<R: 'static>(&self, compute: for<'a, 'b> fn(&#bound_name<'a, 'b>) -> R) -> ::blockui::Live<R> {
                let descriptor = self.__descriptor();
                ::blockui::Live::captured(move |cx| Ok(compute(&descriptor.bind(cx)?)))
            }
            /// Build keyed list rows with a freshly bound receiver and owned key.
            /// Accepts a noncapturing callback; reads execute only when the list builds.
            pub fn live_each<R: 'static>(&self, compute: for<'a, 'b> fn(&#bound_name<'a, 'b>, String) -> R) -> ::blockui::LiveEach<R> {
                let descriptor = self.__descriptor();
                ::blockui::LiveEach::captured(move |cx, key| Ok(compute(&descriptor.bind(cx)?, key)))
            }
            /// Evaluate text with a freshly bound receiver in an independent text observer.
            /// Reads are tracked dynamically; constructing the binding does not read state.
            /// Accepts a noncapturing closure or function returning String or static text.
            /// View-local values and this turn's receiver cannot be captured. Give repeated
            /// text widgets distinct keys. The callback remains in the current implementation.
            pub fn live_text<R: Into<String> + 'static>(&self, compute: for<'a, 'b> fn(&#bound_name<'a, 'b>) -> R) -> ::blockui::Binding {
                let descriptor = self.__descriptor();
                ::blockui::Binding::captured(move |cx| Ok(compute(&descriptor.bind(cx)?).into()))
            }
            /// Evaluate a style with a freshly bound receiver in an independent observer.
            /// Pass this callback to `style_reactive`. Construction does not read state;
            /// each evaluation tracks only the reads of the currently selected branch.
            /// The callback cannot capture this turn's receiver or local values.
            pub fn live_style(&self, compute: for<'a, 'b> fn(&#bound_name<'a, 'b>) -> ::blockui::ViewStyle)
                -> impl for<'a> Fn(&::blockui::Cx<'a>) -> ::blockui::Result<::blockui::ViewStyle> + 'static
            {
                let descriptor = self.__descriptor();
                move |cx| Ok(compute(&descriptor.bind(cx)?))
            }
            #[doc(hidden)]
            pub fn __descriptor(&self) -> #name {
                let mut value = #name::new();
                for scope in self.__actor_identity.scopes() { value = value.scoped(scope); }
                value
            }
            /// Call a typed business operation owned by the stable executable.
            pub fn call<R: ::blockui::ServiceRequest>(&self, request: R) -> ::blockui::Result<R::Response> {
                self.__actor_context.call(request)
            }
        }
        impl ::blockui::BindState for #name {
            type Bound<'cx, 'host> = #bound_name<'cx, 'host> where 'host: 'cx;
            fn bind<'cx, 'host>(&self, cx: &'cx ::blockui::Cx<'host>) -> ::blockui::Result<Self::Bound<'cx, 'host>> {
                Ok(#bound_value)
            }
        }
        impl #name {
            #[doc(hidden)]
            pub const __COMPONENT_ACTIONS: &'static [&'static [::blockui::Action]] = &[#(#component_actions),*];
            #[doc(hidden)]
            pub const __COMPONENT_SCOPES: &'static [&'static [::blockui::Scope]] = &[#(#component_scopes),*];
            #[doc(hidden)]
            pub const __COMPONENT_COMPUTEDS: &'static [&'static [::blockui::ComputedSpec]] = &[#(#component_computeds),*];
            #component_migration
            #record_initialization
            #route_validation
            #[doc(hidden)]
            pub fn __validate_record_codecs(cx: &::blockui::Cx<'_>, previous: u64) -> ::blockui::Result<()> {
                Self::__validate_record_codecs_with_scopes(cx, previous, &[])
            }
            #codec_validation
            pub fn bind<'cx, 'host>(&self, cx: &'cx ::blockui::Cx<'host>) -> ::blockui::Result<#bound_name<'cx, 'host>> {
                ::blockui::BindState::bind(self, cx)
            }
            const __STATE_PARTS: &'static [&'static [::blockui::StateSpec]] = &[#(#specs),*];
            const __STATE_FIELDS: [::blockui::StateSpec; ::blockui::__schema_len(Self::__STATE_PARTS)] = ::blockui::__merge_schema(Self::__STATE_PARTS);
            pub const SCHEMA: &'static [::blockui::StateSpec] = &Self::__STATE_FIELDS;
            pub const fn new() -> Self { Self { #(#initializers),* } }
            #[doc(hidden)]
            pub fn __in_context(cx: &::blockui::Cx<'_>) -> Self {
                let mut value = Self::new();
                for scope in cx.__instance().scopes() { value = value.scoped(scope); }
                value
            }
            pub const fn scoped(self, scope: ::blockui::Key) -> Self { Self { #(#scoped_fields),* } }
        }
        impl ::core::default::Default for #name { fn default() -> Self { Self::new() } }
    })
}

/// Generate descriptors and the Actor implementation from ordinary receiver methods.
#[proc_macro_attribute]
pub fn actor_impl(attributes: TokenStream, input: TokenStream) -> TokenStream {
    sdk_path::resolve(actor_impl_tokens(attributes, input))
}
fn actor_impl_tokens(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let render = attributes.to_string() == "render";
    if !attributes.is_empty() && !render {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "actor_impl accepts only the optional render mode",
        )
        .into_compile_error()
        .into();
    }
    let input: proc_macro2::TokenStream = input.into();
    let mut item = match opaque_impl::parse(input.clone()) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error().into(),
    };
    match expand_actor_impl_mode(&mut item, render) {
        Ok(tokens) => tokens.into(),
        Err(error) => {
            let original = opaque_impl::parse(input).expect("previously parsed actor impl");
            recover_actor_impl(&original, render, error).into()
        }
    }
}

// Retry only erroneous input. Omitting malformed annotated methods can
// preserve other accessors for tooling. The original diagnostic still rejects
// compilation. Always retry from pristine input, never partial failed output.
fn recover_actor_impl(
    original: &syn::ItemImpl,
    render: bool,
    error: syn::Error,
) -> proc_macro2::TokenStream {
    let diagnostic = error.into_compile_error();
    let annotated: Vec<usize> = original
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, member)| {
            let syn::ImplItem::Fn(method) = member else {
                return None;
            };
            method
                .attrs
                .iter()
                .any(|attr| {
                    ["scope", "computed", "action"]
                        .iter()
                        .any(|role| attr.path().is_ident(role))
                })
                .then_some(index)
        })
        .collect();
    if let Some(&index) = annotated.first() {
        let mut candidate = original.clone();
        candidate.items.remove(index);
        if let Ok(expanded) = expand_actor_impl_mode(&mut candidate, render) {
            return quote!(#diagnostic #expanded);
        }
    }
    // Validate declarations independently before retrying whole-impl omissions.
    // Keep ordinary methods (including the view and accessor conflicts) in every
    // probe. Publish recovery output only if removing all bad declarations makes
    // the remaining implementation pass the normal expansion checks.
    // Clone ordinary items once; insert each declaration at its authored position.
    let mut probe_template = original.clone();
    for &index in annotated.iter().rev() {
        probe_template.items.remove(index);
    }
    let mut invalid = Vec::new();
    for (removed_before, &index) in annotated.iter().enumerate() {
        let mut probe = probe_template.clone();
        probe
            .items
            .insert(index - removed_before, original.items[index].clone());
        if let Err(error) = expand_actor_impl_mode(&mut probe, render) {
            invalid.push((index, error));
        }
    }
    if !invalid.is_empty() {
        let mut candidate = original.clone();
        candidate.items = original
            .items
            .iter()
            .enumerate()
            .filter(|(index, _)| !invalid.iter().any(|(bad, _)| bad == index))
            .map(|(_, member)| member.clone())
            .collect();
        if let Ok(expanded) = expand_actor_impl_mode(&mut candidate, render) {
            let diagnostics = if invalid.len() == 1 {
                diagnostic.clone()
            } else {
                let errors = invalid
                    .into_iter()
                    .map(|(_, error)| error.into_compile_error());
                quote!(#(#errors)*)
            };
            return quote!(#diagnostics #expanded);
        }
    }
    // Cross-method conflicts can disappear only when one valid method is omitted.
    for &index in annotated.iter().skip(1) {
        let mut candidate = original.clone();
        candidate.items.remove(index);
        if let Ok(expanded) = expand_actor_impl_mode(&mut candidate, render) {
            return quote!(#diagnostic #expanded);
        }
    }
    diagnostic
}
#[cfg(test)]
fn expand_actor_impl(item: &mut syn::ItemImpl) -> syn::Result<proc_macro2::TokenStream> {
    expand_actor_impl_mode(item, false)
}
fn expand_actor_impl_mode(
    item: &mut syn::ItemImpl,
    render: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    expand_implementation_mode(item, render, false)
}
fn expand_implementation_mode(
    item: &mut syn::ItemImpl,
    render: bool,
    component: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    expand_implementation_kind(item, render, component, false)
}
fn expand_implementation_kind(
    item: &mut syn::ItemImpl,
    render: bool,
    component: bool,
    app_root: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    if item.trait_.is_some() || !item.generics.params.is_empty() {
        return Err(syn::Error::new(
            item.span(),
            "actor_impl requires a nongeneric inherent impl",
        ));
    }
    normalize_type(&mut item.self_ty);
    let actor = &item.self_ty;
    let mut authored_names = std::collections::BTreeMap::new();
    for member in &item.items {
        let (name, attrs) = match member {
            syn::ImplItem::Fn(method)
                if !method.attrs.iter().any(|attr| {
                    ["action", "computed", "scope"]
                        .iter()
                        .any(|role| attr.path().is_ident(role))
                }) =>
            {
                (&method.sig.ident, &method.attrs)
            }
            syn::ImplItem::Const(item) => (&item.ident, &item.attrs),
            _ => continue,
        };
        if conditional_attributes(attrs)?.is_empty() {
            authored_names.insert(name.to_string(), name.span());
        }
    }
    let check_accessor = |name: &syn::Ident, attrs: &[syn::Attribute]| -> syn::Result<()> {
        if attrs.is_empty()
            && let Some(previous) = authored_names.get(&name.to_string())
        {
            let mut error = syn::Error::new(
                name.span(),
                format!(
                    "generated accessor `{name}` conflicts with an existing associated item; rename that item or the annotated method"
                ),
            );
            error.combine(syn::Error::new(
                *previous,
                "conflicting associated item is declared here",
            ));
            return Err(error);
        }
        Ok(())
    };
    let mut accessors = Vec::new();
    let mut bound_accessors = Vec::new();
    let mut bound_methods = Vec::new();
    let mut moved_methods = std::collections::BTreeSet::new();
    let mut actions = Vec::new();
    let mut action_fields = Vec::new();
    let mut action_values = Vec::new();
    let mut bound_action_values = Vec::new();
    let mut explicit_action_names = std::collections::BTreeMap::new();
    let mut scopes = Vec::new();
    let mut explicit_scope_names = std::collections::BTreeMap::new();
    let mut computeds = Vec::new();
    let mut explicit_computed_names = std::collections::BTreeMap::new();
    let mut migration = Vec::new();
    let mut has_view = false;
    let mut component_renderers = Vec::new();
    for (member_index, member) in item.items.iter_mut().enumerate() {
        let syn::ImplItem::Fn(method) = member else {
            continue;
        };
        for argument in &mut method.sig.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                normalize_type(&mut argument.ty);
            }
        }
        if let syn::ReturnType::Type(_, output) = &mut method.sig.output {
            normalize_type(output);
        }
        let role_attributes: Vec<_> = method
            .attrs
            .iter()
            .filter(|attr| {
                attr.path().is_ident("action")
                    || attr.path().is_ident("computed")
                    || attr.path().is_ident("scope")
            })
            .collect();
        if role_attributes.len() > 1 {
            let mut error = syn::Error::new(
                role_attributes[1].span(),
                "a method can have only one action, computed or scope declaration",
            );
            error.combine(syn::Error::new(
                role_attributes[0].span(),
                "first method role is declared here; keep only one role annotation",
            ));
            return Err(error);
        }
        if matches!(method.sig.ident.to_string().as_str(), "view" | "migrate")
            && let Some(attribute) = role_attributes.first()
        {
            return Err(syn::Error::new(
                attribute.span(),
                "view and migrate are lifecycle methods; declare actions, computeds and scopes on separate methods",
            ));
        }
        if component && method.sig.ident == "view" {
            let explicit_builder = method.attrs.iter().any(|attribute| {
                attribute
                    .path()
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "view")
                    && matches!(attribute.meta, syn::Meta::Path(_))
            });
            let expected_inputs = if explicit_builder {
                view_blocks::declared_builder(&method.sig)?;
                2
            } else {
                1
            };
            let receiver = method.sig.receiver();
            if method.sig.asyncness.is_some()
                || method.sig.unsafety.is_some()
                || !method.sig.generics.params.is_empty()
                || receiver.is_none_or(|r| r.reference.is_none() || r.mutability.is_some())
                || method.sig.inputs.len() != expected_inputs
            {
                return Err(syn::Error::new(
                    method.sig.span(),
                    if explicit_builder {
                        "component view with #[view] requires synchronous fn view(&self, ui: &Ui) -> View or Result<View>"
                    } else {
                        "component view requires synchronous fn view(&self) -> View or Result<View>"
                    },
                ));
            }
            let syn::ReturnType::Type(_, output) = &method.sig.output else {
                return Err(syn::Error::new(
                    method.sig.span(),
                    "component view requires View or Result<View>",
                ));
            };
            let output = output.clone();
            let cfg = conditional_attributes(&method.attrs)?;
            let mut body = method.clone();
            body.sig.ident = syn::Ident::new("__render_body", method.sig.ident.span());
            body.vis = syn::Visibility::Inherited;
            bound_methods.push(quote!(#body));
            let component_output = quote::quote_spanned!(
                syn::spanned::Spanned::span(&output)=>
                ::blockui::__component_view(self.__render_body(), self.__actor_identity,
                    ::blockui::Key::named(concat!("component.", stringify!(#actor))))
            );
            let renderer: syn::ImplItemFn = syn::parse_quote! {
                fn render(&self) -> #output { #component_output }
            };
            component_renderers.push((cfg, output, renderer));
            moved_methods.insert(member_index);
            continue;
        }
        if method.sig.ident == "view" {
            let explicit_builder = method.attrs.iter().any(|attribute| {
                attribute
                    .path()
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "view")
                    && matches!(attribute.meta, syn::Meta::Path(_))
            });
            if explicit_builder {
                view_blocks::declared_builder(&method.sig)?;
            }
            if matches!(method.sig.output, syn::ReturnType::Default) {
                return Err(syn::Error::new(
                    method.sig.span(),
                    "actor view requires an explicit View or Result<View> return type",
                ));
            }
            let expected_inputs = if explicit_builder { 3 } else { 2 };
            let receiver = method.sig.receiver();
            if method.sig.asyncness.is_some()
                || method.sig.unsafety.is_some()
                || !method.sig.generics.params.is_empty()
                || receiver.is_none_or(|receiver| {
                    receiver.reference.is_none() || receiver.mutability.is_some()
                })
                || method.sig.inputs.len() != expected_inputs
            {
                return Err(syn::Error::new(
                    method.sig.span(),
                    if explicit_builder {
                        "view with #[view] requires synchronous fn view(&self, cx: &Cx<'_>, ui: &Ui) -> View or Result<View>"
                    } else {
                        "view requires synchronous fn view(&self, cx: &Cx<'_>) -> View or Result<View>"
                    },
                ));
            }
            has_view = true;
        }
        if method.sig.ident == "migrate" {
            let receiver = method.sig.receiver();
            if method.sig.asyncness.is_some()
                || method.sig.unsafety.is_some()
                || !method.sig.generics.params.is_empty()
                || receiver.is_none_or(|receiver| {
                    receiver.reference.is_none() || receiver.mutability.is_some()
                })
                || method.sig.inputs.len() != 3
            {
                return Err(syn::Error::new(
                    method.sig.span(),
                    "migration requires fn migrate(&self, cx: &Cx<'_>, previous_schema: u64) -> Result<()>",
                ));
            }
            let attrs = conditional_attributes(&method.attrs)?;
            migration.push(quote! {
                #(#attrs)*
                {
                    migration = Some(|cx, previous| Self::__in_context(cx).migrate(cx, previous));
                }
            });
        }
        if let Some(index) = method
            .attrs
            .iter()
            .position(|attr| attr.path().is_ident("scope"))
        {
            let attribute = method.attrs.remove(index);
            let mut name = None;
            let mut bound = false;
            if let syn::Meta::List(_) = &attribute.meta {
                attribute.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") && name.is_none() {
                        name = Some(meta.value()?.parse::<LitStr>()?);
                        Ok(())
                    } else if meta.path.is_ident("bound") && !bound {
                        bound = true;
                        Ok(())
                    } else {
                        Err(meta.error("expected bound or name = \"stable.scope.name\""))
                    }
                })?;
            } else if !matches!(attribute.meta, syn::Meta::Path(_)) {
                return Err(syn::Error::new(
                    attribute.span(),
                    "scope accepts only bound and an optional name",
                ));
            }
            if bound && !render {
                return Err(syn::Error::new(
                    attribute.span(),
                    "bound scopes require actor_impl(render)",
                ));
            }
            let explicit_builder = method.attrs.iter().any(|attribute| {
                attribute
                    .path()
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "view")
                    && matches!(attribute.meta, syn::Meta::Path(_))
            });
            if explicit_builder {
                view_blocks::declared_builder(&method.sig)?;
            }
            if matches!(method.sig.output, syn::ReturnType::Default) {
                return Err(syn::Error::new(
                    method.sig.span(),
                    "scope requires an explicit View or Result<View> return type",
                ));
            }
            let expected_inputs = if bound { 1 } else { 2 } + usize::from(explicit_builder);
            let receiver = method.sig.receiver();
            if method.sig.asyncness.is_some()
                || method.sig.unsafety.is_some()
                || !method.sig.generics.params.is_empty()
                || receiver.is_none_or(|receiver| {
                    receiver.reference.is_none() || receiver.mutability.is_some()
                })
                || method.sig.inputs.len() != expected_inputs
            {
                return Err(syn::Error::new(
                    method.sig.span(),
                    match (bound, explicit_builder, render) {
                        (true, true, _) => {
                            "bound scope with #[view] requires synchronous fn method(&self, ui: &Ui) -> View or Result<View>"
                        }
                        (false, true, true) => {
                            "scope with #[view] requires synchronous fn method(&self, cx: &Cx<'_>, ui: &Ui) -> View or Result<View>; use #[scope(bound)] for a synchronous method(&self, ui: &Ui) on the turn-bound receiver"
                        }
                        (false, true, false) => {
                            "scope with #[view] requires synchronous fn method(&self, cx: &Cx<'_>, ui: &Ui) -> View or Result<View>"
                        }
                        (true, false, _) => {
                            "bound scope requires synchronous fn method(&self) -> View or Result<View>"
                        }
                        (false, false, true) => {
                            "scope requires synchronous fn method(&self, cx: &Cx<'_>) -> View or Result<View>; use #[scope(bound)] for a synchronous method(&self) on the turn-bound receiver"
                        }
                        (false, false, false) => {
                            "scope requires synchronous fn method(&self, cx: &Cx<'_>) -> View or Result<View>"
                        }
                    },
                ));
            }
            let method_name = &method.sig.ident;
            let accessor = quote::format_ident!("{}_scope", method_name);
            let attrs = conditional_attributes(&method.attrs)?;
            check_accessor(&accessor, &attrs)?;
            let resolved_name = name
                .as_ref()
                .map(LitStr::value)
                .unwrap_or_else(|| format!("{}.{}", quote!(#actor), method_name));
            if resolved_name.is_empty() {
                return Err(syn::Error::new(
                    attribute.span(),
                    "scope name cannot be empty",
                ));
            }
            if attrs.is_empty()
                && let Some(previous) =
                    explicit_scope_names.insert(resolved_name.clone(), attribute.span())
            {
                let mut error = syn::Error::new(
                    attribute.span(),
                    format!(
                        "duplicate scope name `{resolved_name}`; each structural boundary needs a distinct name"
                    ),
                );
                error.combine(syn::Error::new(
                    previous,
                    "first scope with this name is here",
                ));
                return Err(error);
            }
            let identity = match name {
                Some(name) => quote!(#name),
                None => quote!(concat!(stringify!(#actor), ".", stringify!(#method_name))),
            };
            let scope_docs: Vec<_> = method
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("doc"))
                .collect();
            let scope_call = if bound {
                moved_methods.insert(member_index);
                bound_methods.push(quote!(#method));
                quote::quote_spanned!(syn::spanned::Spanned::span(&method.sig.output)=> Self::__in_context(cx).bind(cx)?.#method_name())
            } else {
                quote::quote_spanned!(syn::spanned::Spanned::span(&method.sig.output)=> Self::__in_context(cx).#method_name(cx))
            };
            let scope_output = quote::quote_spanned!(
                syn::spanned::Spanned::span(&method.sig.output)=>
                ::blockui::__view_result(#scope_call)
            );
            accessors.push(quote! {
                #(#attrs)*
                #[doc = "Return a structural scope descriptor to mount in the parent view. State reads are tracked dynamically when its builder runs; obtaining this descriptor does not run the builder. Calling the original method directly executes it in the caller's observer instead of mounting this boundary. Use independent text/style bindings for presentation-only updates.\n\n"]
                #(#scope_docs)*
                pub const fn #accessor() -> ::blockui::Scope {
                    ::blockui::Scope::new(#identity, |cx| #scope_output)
                }
            });
            if render {
                bound_accessors.push(quote! {
                    #(#attrs)*
                    #[doc = "Return this instance's structural scope descriptor to mount in the parent view. State reads are tracked dynamically when its builder runs; obtaining this descriptor does not run the builder. Calling the original method directly executes it in the caller's observer instead of mounting this boundary. Use independent text/style bindings for presentation-only updates.\n\n"]
                    #(#scope_docs)*
                    pub fn #accessor(&self) -> ::blockui::Scope {
                        <#actor>::#accessor().__instance(self.__actor_identity)
                    }
                });
            }
            scopes.push(quote!(#(#attrs)* Self::#accessor()));
            continue;
        }
        if let Some(index) = method
            .attrs
            .iter()
            .position(|attr| attr.path().is_ident("computed"))
        {
            let attribute = method.attrs.remove(index);
            let mut name = None::<LitStr>;
            let mut bound = false;
            let mut record = false;
            if !matches!(attribute.meta, syn::Meta::Path(_)) {
                attribute.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") && name.is_none() {
                        name = Some(meta.value()?.parse()?);
                        Ok(())
                    } else if meta.path.is_ident("bound") && !bound {
                        bound = true;
                        Ok(())
                    } else if meta.path.is_ident("record") && !record {
                        record = true;
                        Ok(())
                    } else {
                        Err(meta.error("expected record, bound or name = \"stable.computed.name\""))
                    }
                })?;
            }
            if bound && !render {
                return Err(syn::Error::new(
                    attribute.span(),
                    "bound computed requires actor_impl(render)",
                ));
            }
            let receiver = method.sig.receiver();
            if method.sig.asyncness.is_some()
                || method.sig.unsafety.is_some()
                || !method.sig.generics.params.is_empty()
                || receiver.is_none_or(|receiver| {
                    receiver.reference.is_none() || receiver.mutability.is_some()
                })
                || method.sig.inputs.len() != if bound { 1 } else { 2 }
            {
                return Err(syn::Error::new(
                    method.sig.span(),
                    if bound {
                        "bound computed requires synchronous fn method(&self) -> T or Result<T>"
                    } else if render {
                        "computed requires fn method(&self, cx: &Cx<'_>) -> T or Result<T>; use #[computed(bound)] for a synchronous method(&self) on the turn-bound receiver"
                    } else {
                        "computed requires fn method(&self, cx: &Cx<'_>) -> T or Result<T>"
                    },
                ));
            }
            let syn::ReturnType::Type(_, output) = &method.sig.output else {
                return Err(syn::Error::new(
                    method.sig.span(),
                    "computed requires an explicit return type T or Result<T>",
                ));
            };
            let (value, fallible) = if let Type::Path(path) = output.as_ref()
                && let Some(segment) = path.path.segments.last()
                && segment.ident == "Result"
            {
                let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return Err(syn::Error::new(
                        output.span(),
                        "computed Result requires a value type and optional error type",
                    ));
                };
                let Some(GenericArgument::Type(value)) = arguments.args.first() else {
                    return Err(syn::Error::new(
                        output.span(),
                        "computed Result requires a value type and optional error type",
                    ));
                };
                if !(1..=2).contains(&arguments.args.len())
                    || arguments
                        .args
                        .iter()
                        .any(|argument| !matches!(argument, GenericArgument::Type(_)))
                {
                    return Err(syn::Error::new(
                        output.span(),
                        "computed Result requires a value type and optional error type",
                    ));
                }
                (value, true)
            } else {
                (output.as_ref(), false)
            };
            let opaque_output = !record
                && !fallible
                && !matches!(output.as_ref(), Type::Path(path)
                if path.path.segments.last().is_some_and(|segment|
                    matches!(segment.ident.to_string().as_str(), "i64" | "f64" | "bool" | "String" | "Vec")));
            let value = if opaque_output {
                quote!(<#output as ::blockui::__ComputedOutput>::Value)
            } else {
                quote!(#value)
            };
            let computed_type = if record {
                quote!(::blockui::RecordComputed<#value>)
            } else {
                quote!(::blockui::Computed<#value>)
            };
            let method_name = method.sig.ident.clone();
            let body_name = quote::format_ident!("__compute_{}", method_name);
            let visibility = &method.vis;
            method.sig.ident = body_name.clone();
            let accessor = quote::format_ident!("{}_computed", method_name);
            let attrs = conditional_attributes(&method.attrs)?;
            check_accessor(&accessor, &attrs)?;
            let resolved_name = name.as_ref().map(LitStr::value).or_else(|| {
                let Type::Path(path) = actor.as_ref() else {
                    return None;
                };
                (path.qself.is_none() && path.path.segments.len() == 1)
                    .then(|| format!("{}.{}", path.path.segments[0].ident, method_name))
            });
            if attrs.is_empty()
                && let Some(resolved_name) = resolved_name
                && let Some(previous) = explicit_computed_names.insert(
                    resolved_name.clone(),
                    name.as_ref().map_or(method_name.span(), LitStr::span),
                )
            {
                let (span, message) = match &name {
                    Some(name) => (name.span(), "duplicate explicit computed name"),
                    None => (method_name.span(), "duplicate generated computed name"),
                };
                let mut error = syn::Error::new(span, format!("{message} `{resolved_name}`"));
                error.combine(syn::Error::new(
                    previous,
                    "first computed with this name is here",
                ));
                return Err(error);
            }
            let identity = match name {
                Some(name) if !name.value().is_empty() => quote!(#name),
                Some(name) => {
                    return Err(syn::Error::new(
                        name.span(),
                        "computed name cannot be empty",
                    ));
                }
                None => quote!(concat!(stringify!(#actor), ".", stringify!(#method_name))),
            };
            let computed_docs: Vec<_> = method
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("doc"))
                .collect();
            let finish_read = if fallible {
                quote!()
            } else {
                quote!(.expect("infallible computed evaluation failed"))
            };
            let failure_docs = if opaque_output {
                quote!(#[doc = "If the return alias is infallible, panics on evaluation failure; a Result alias preserves the error."])
            } else if fallible {
                quote!()
            } else {
                quote!(#[doc = "Panics if computed evaluation fails. Return Result<T> when callers should handle evaluation errors."])
            };
            let read = quote::quote_spanned!(output.span()=> Self::#accessor().get(cx));
            let bound_read = quote::quote_spanned!(output.span()=> <#actor>::#accessor().__instance(self.__actor_identity).get(self.__actor_context));
            let read = if opaque_output {
                quote::quote_spanned!(output.span()=>
                    <#output as ::blockui::__ComputedOutput>::read(Self::#accessor(), cx)
                )
            } else {
                quote!(#read #finish_read)
            };
            let bound_read = if opaque_output {
                quote::quote_spanned!(output.span()=>
                    <#output as ::blockui::__ComputedOutput>::read(
                        <#actor>::#accessor().__instance(self.__actor_identity), self.__actor_context
                    )
                )
            } else {
                quote!(#bound_read #finish_read)
            };
            if render {
                bound_accessors.push(quote! {
                    #(#attrs)*
                    #[doc = "Return this instance's computed descriptor without evaluating it.\n\n"]
                    #(#computed_docs)*
                    pub fn #accessor(&self) -> #computed_type {
                        <#actor>::#accessor().__instance(self.__actor_identity)
                    }
                    #(#attrs)*
                    #[doc = "Read the cached computed value, evaluating it when dirty and tracking the read for the current observer.\n\n"]
                    #(#computed_docs)*
                    #failure_docs
                    #visibility fn #method_name(&self) -> #output {
                        #bound_read
                    }
                });
            }
            let compute_call = if bound {
                moved_methods.insert(member_index);
                bound_methods.push(quote!(#method));
                quote!(Self::__in_context(cx).bind(cx)?.#body_name())
            } else {
                quote!(Self::__in_context(cx).#body_name(cx))
            };
            let compute_call = if opaque_output {
                quote!(<#output as ::blockui::__ComputedOutput>::into_result(#compute_call))
            } else if fallible {
                compute_call
            } else {
                quote!(Ok(#compute_call))
            };
            let constructor = if record {
                quote::quote_spanned!(output.span()=>
                    ::blockui::RecordComputed::<#value>::__encoded(#identity, |cx| {
                        let value: ::blockui::Result<#value> = #compute_call;
                        let value = value?;
                        ::blockui::record_codec::encode_record(&value)
                    })
                )
            } else {
                quote::quote_spanned!(output.span()=>
                    ::blockui::Computed::<#value>::new(#identity, |cx| #compute_call)
                )
            };
            accessors.push(quote! { #(#attrs)*
            #[doc = "Return the computed descriptor without evaluating it.\n\n"]
            #(#computed_docs)*
            pub const fn #accessor() -> #computed_type {
                #constructor
            }
            #(#attrs)*
            #[doc = "Read the cached computed value, evaluating it when dirty and tracking the read for the current observer.\n\n"]
            #(#computed_docs)*
            #failure_docs
            #visibility fn #method_name(&self, cx: &::blockui::Cx<'_>) -> #output {
                #read
            } });
            computeds.push(quote!(#(#attrs)* Self::#accessor().spec()));
            continue;
        }
        let Some(index) = method
            .attrs
            .iter()
            .position(|attr| attr.path().is_ident("action"))
        else {
            // Explicit UI helpers execute with the same turn-bound receiver as
            // the lifecycle view. Their view macro owns body recovery and any
            // parameters; ordinary descriptor helpers retain their old meaning.
            if component
                && method.sig.receiver().is_some()
                && method.attrs.iter().any(|attr| {
                    attr.path()
                        .segments
                        .last()
                        .is_some_and(|segment| segment.ident == "view")
                })
            {
                moved_methods.insert(member_index);
                bound_methods.push(quote!(#method));
            }
            continue;
        };
        let attribute = method.attrs.remove(index);
        let mut name = None::<LitStr>;
        let mut bound = false;
        if !matches!(attribute.meta, syn::Meta::Path(_)) {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") && name.is_none() {
                    name = Some(meta.value()?.parse()?);
                    Ok(())
                } else if meta.path.is_ident("bound") && !bound {
                    bound = true;
                    Ok(())
                } else {
                    Err(meta.error("expected bound or name = \"stable.action.name\""))
                }
            })?;
        }
        let receiver = method.sig.receiver();
        if method.sig.asyncness.is_some()
            || method.sig.unsafety.is_some()
            || !method.sig.generics.params.is_empty()
            || receiver.is_none_or(|receiver| {
                receiver.reference.is_none() || receiver.mutability.is_some()
            })
            || if bound {
                !(1..=2).contains(&method.sig.inputs.len())
            } else {
                !(2..=3).contains(&method.sig.inputs.len())
            }
        {
            return Err(syn::Error::new(
                method.sig.span(),
                if bound {
                    "bound action requires a synchronous fn method(&self[, value]) returning Result; omit the explicit Cx argument"
                } else if render {
                    "action requires fn method(&self, cx: &Cx<'_>[, value]) returning Result; use #[action(bound)] for a synchronous method(&self[, value]) on the turn-bound receiver"
                } else {
                    "action requires fn method(&self, cx: &Cx<'_>[, value: i64/bool/f64/String/LocalPoint/RequestError]) -> Result<()> or Result<i64/bool/f64>"
                },
            ));
        }
        let method_name = &method.sig.ident;
        let accessor = quote::format_ident!("{}_action", method_name);
        let attrs = conditional_attributes(&method.attrs)?;
        check_accessor(&accessor, &attrs)?;
        let resolved_name = name.as_ref().map(LitStr::value).or_else(|| {
            let Type::Path(path) = actor.as_ref() else {
                return None;
            };
            (path.qself.is_none() && path.path.segments.len() == 1)
                .then(|| format!("{}.{}", path.path.segments[0].ident, method_name))
        });
        if attrs.is_empty()
            && let Some(resolved_name) = resolved_name
            && let Some(previous) = explicit_action_names.insert(
                resolved_name.clone(),
                name.as_ref().map_or(method_name.span(), LitStr::span),
            )
        {
            let (span, message) = match &name {
                Some(name) => (name.span(), "duplicate explicit action name"),
                None => (method_name.span(), "duplicate generated action name"),
            };
            let mut error = syn::Error::new(span, format!("{message} `{resolved_name}`"));
            error.combine(syn::Error::new(
                previous,
                "first action with this name is here",
            ));
            return Err(error);
        }
        let identity = match name {
            Some(name) if !name.value().is_empty() => quote!(#name),
            Some(name) => return Err(syn::Error::new(name.span(), "action name cannot be empty")),
            None => quote!(concat!(stringify!(#actor), ".", stringify!(#method_name))),
        };
        if bound && !render {
            return Err(syn::Error::new(
                attribute.span(),
                "bound actions require actor_impl(render)",
            ));
        }
        let has_payload = method.sig.inputs.len() == if bound { 2 } else { 3 };
        let call = if bound {
            moved_methods.insert(member_index);
            bound_methods.push(quote!(#method));
            if has_payload {
                quote!(Self::__in_context(cx).bind(cx)?.#method_name(value))
            } else {
                quote!(Self::__in_context(cx).bind(cx)?.#method_name())
            }
        } else if has_payload {
            quote!(Self::__in_context(cx).#method_name(cx, value))
        } else {
            quote!(Self::__in_context(cx).#method_name(cx))
        };
        let mut request_error = false;
        let mut text_payload = false;
        let mut point_payload = false;
        let mut keyed_text_payload = false;
        let mut payload_type = quote!(i64);
        let typed_kind = match method.sig.inputs.last() {
            Some(syn::FnArg::Typed(value)) if has_payload => {
                let mut ty = value.ty.as_ref();
                loop {
                    ty = match ty {
                        Type::Paren(inner) => &inner.elem,
                        Type::Group(inner) => &inner.elem,
                        _ => break,
                    };
                }
                let Type::Path(path) = ty else {
                    return Err(syn::Error::new(
                        ty.span(),
                        "action payload must be i64, bool, f64, String, KeyedText, LocalPoint or RequestError; use the SDK for other adapters",
                    ));
                };
                let names: Vec<_> = path
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect();
                let qualified_error = path.qself.is_none()
                    && matches!(
                        names
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .as_slice(),
                        ["blockui", "RequestError"]
                            | ["blockui", "abi", "RequestError"]
                            | ["blockui_abi", "RequestError"]
                    )
                    && path
                        .path
                        .segments
                        .iter()
                        .all(|segment| matches!(segment.arguments, PathArguments::None));
                let qualified_string = path.qself.is_none()
                    && matches!(
                        names
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .as_slice(),
                        ["String"] | ["std", "string", "String"] | ["alloc", "string", "String"]
                    )
                    && path
                        .path
                        .segments
                        .iter()
                        .all(|segment| matches!(segment.arguments, PathArguments::None));
                let qualified_keyed_text = path.qself.is_none()
                    && matches!(
                        names
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .as_slice(),
                        ["KeyedText"] | ["blockui", "KeyedText"]
                    )
                    && path
                        .path
                        .segments
                        .iter()
                        .all(|segment| matches!(segment.arguments, PathArguments::None));
                let qualified_point = path.qself.is_none()
                    && matches!(
                        names
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .as_slice(),
                        ["LocalPoint"] | ["blockui", "LocalPoint"]
                    )
                    && path
                        .path
                        .segments
                        .iter()
                        .all(|segment| matches!(segment.arguments, PathArguments::None));
                let primitive = if qualified_keyed_text {
                    Some("KeyedText")
                } else if qualified_point {
                    Some("LocalPoint")
                } else if qualified_string {
                    Some("String")
                } else if qualified_error {
                    Some("RequestError")
                } else if path.qself.is_none()
                    && (names.len() == 1
                        || (names.len() == 3
                            && matches!(names[0].as_str(), "core" | "std")
                            && names[1] == "primitive"))
                    && path
                        .path
                        .segments
                        .iter()
                        .all(|segment| matches!(segment.arguments, PathArguments::None))
                {
                    names.last().map(String::as_str)
                } else {
                    None
                };
                match primitive {
                    Some("KeyedText") if qualified_keyed_text => {
                        keyed_text_payload = true;
                        payload_type = quote!(::blockui::KeyedText);
                        Some(quote!(::blockui::KeyedTextAction))
                    }
                    Some("LocalPoint") if qualified_point => {
                        point_payload = true;
                        payload_type = quote!(::blockui::LocalPoint);
                        Some(quote!(::blockui::PointAction))
                    }
                    Some("bool") => {
                        payload_type = quote!(bool);
                        Some(quote!(::blockui::BoolAction))
                    }
                    Some("f64") => {
                        payload_type = quote!(f64);
                        Some(quote!(::blockui::FloatAction))
                    }
                    Some("String") if qualified_string => {
                        text_payload = true;
                        payload_type = quote!(::std::string::String);
                        Some(quote!(::blockui::TextAction))
                    }
                    Some("i64") => None,
                    Some("RequestError") => {
                        request_error = true;
                        None
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ty.span(),
                            "action payload must be i64, bool, f64, String, KeyedText, LocalPoint or RequestError; type aliases cannot be resolved by this macro",
                        ));
                    }
                }
            }
            _ => None,
        };
        let result_error = || {
            syn::Error::new(
                method.sig.output.span(),
                "action result must be Result<()> or Result<i64/bool/f64/String>; aliases cannot be resolved",
            )
        };
        let syn::ReturnType::Type(_, output) = &method.sig.output else {
            return Err(result_error());
        };
        let Type::Path(output) = output.as_ref() else {
            return Err(result_error());
        };
        let Some(segment) = output.path.segments.last() else {
            return Err(result_error());
        };
        let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return Err(result_error());
        };
        if segment.ident != "Result" || !(1..=2).contains(&arguments.args.len()) {
            return Err(result_error());
        }
        let Some(syn::GenericArgument::Type(result)) = arguments.args.first() else {
            return Err(result_error());
        };
        let mut text_reply = false;
        let reply_type = match result {
            Type::Tuple(tuple) if tuple.elems.is_empty() => None,
            Type::Path(path) if path.qself.is_none() => {
                let names: Vec<_> = path
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect();
                let qualified_string = matches!(
                    names
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    ["String"] | ["std", "string", "String"] | ["alloc", "string", "String"]
                );
                if !(qualified_string
                    || names.len() == 1
                    || (names.len() == 3
                        && matches!(names[0].as_str(), "core" | "std")
                        && names[1] == "primitive"))
                    || path
                        .path
                        .segments
                        .iter()
                        .any(|segment| !matches!(segment.arguments, PathArguments::None))
                {
                    return Err(result_error());
                }
                match names.last().map(String::as_str) {
                    Some("i64") => Some(quote!(i64)),
                    Some("bool") => Some(quote!(bool)),
                    Some("f64") => Some(quote!(f64)),
                    Some("String") if qualified_string => {
                        text_reply = true;
                        Some(quote!(::std::string::String))
                    }
                    _ => return Err(result_error()),
                }
            }
            _ => return Err(result_error()),
        };
        if keyed_text_payload && reply_type.is_some() {
            return Err(syn::Error::new(
                method.sig.output.span(),
                "keyed text actions must return Result<()>",
            ));
        }
        if point_payload && reply_type.is_some() {
            return Err(syn::Error::new(
                method.sig.output.span(),
                "point actions must return Result<()>",
            ));
        }
        if (text_payload && reply_type.is_some() && !text_reply) || (text_reply && !text_payload) {
            return Err(syn::Error::new(
                method.sig.output.span(),
                "text requests require String input and String reply; mixed scalar/text requests are unsupported",
            ));
        }
        if request_error && reply_type.is_some() {
            return Err(syn::Error::new(
                method.sig.output.span(),
                "request-error actions must return Result<()>",
            ));
        }
        let registration = if typed_kind.is_some() || reply_type.is_some() {
            quote!(Self::#accessor().spec())
        } else {
            quote!(Self::#accessor())
        };
        let token = if text_reply {
            quote!(::blockui::TextAction)
        } else if let Some(reply) = &reply_type {
            quote!(::blockui::RequestAction<#payload_type, #reply>)
        } else {
            typed_kind.unwrap_or_else(|| quote!(::blockui::Action))
        };
        let action_docs: Vec<_> = method
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("doc"))
            .collect();
        action_fields.push(quote! {
            #(#attrs)*
            #[doc = "Typed action descriptor; reading this field does not invoke the handler.\n\n"]
            #(#action_docs)*
            pub #method_name: #token
        });
        action_values.push(quote!(#(#attrs)* #method_name: <#actor>::#accessor()));
        bound_action_values.push(quote!(#(#attrs)* #method_name: <#actor>::#accessor().__instance(self.__actor_identity)));
        if render {
            bound_accessors.push(quote! {
                #(#attrs)*
                #[doc = "Return this instance's typed action descriptor for dispatch.\n\n"]
                #(#action_docs)*
                pub fn #accessor(&self) -> #token {
                    <#actor>::#accessor().__instance(self.__actor_identity)
                }
            });
        }
        let constructor = if request_error {
            quote!(::blockui::Action::request_error)
        } else {
            quote!(#token::new)
        };
        let adapter = if text_reply {
            quote!(::blockui::TextAction::request(#identity, |cx, value| { let _ = value; #call }))
        } else if let Some(reply) = &reply_type {
            quote!(
                ::blockui::request!(#payload_type => #reply, #identity, |cx, value| { let _ = value; #call })
            )
        } else {
            quote!(#constructor(#identity, |cx, value| { let _ = value; #call }))
        };
        accessors.push(quote! {
            #(#attrs)*
            #[doc = "Return the typed action descriptor for dispatch through the current implementation.\n\n"]
            #(#action_docs)*
            pub const fn #accessor() -> #token {
                #adapter
            }
        });
        actions.push(quote!(#(#attrs)* #registration));
    }
    if render && has_view {
        return Err(syn::Error::new(
            item.span(),
            "render mode uses Render on the bound receiver; remove the explicit view method",
        ));
    }
    if !has_view && !render {
        return Err(syn::Error::new(
            item.span(),
            "actor_impl requires a view(&self, cx: &Cx<'_>) method",
        ));
    }
    let action_group = if action_fields.is_empty() {
        quote!()
    } else {
        if let Some(method) = item.items.iter().find_map(|member| match member {
            syn::ImplItem::Fn(method) if method.sig.ident == "actions" => Some(method),
            _ => None,
        }) {
            return Err(syn::Error::new(
                method.sig.ident.span(),
                "actor_impl reserves actions() for typed action tokens; rename this method and retain its explicit action name",
            ));
        }
        let Type::Path(path) = actor.as_ref() else {
            return Err(syn::Error::new(
                actor.span(),
                "actor actions require a named type",
            ));
        };
        let name = &path.path.segments.last().unwrap().ident;
        let group = quote::format_ident!("{}Actions", name);
        if render {
            bound_accessors.push(
                quote!(pub fn events(&self) -> #group { #group { #(#bound_action_values,)* } }),
            );
        }
        quote! {
            /// Typed action descriptors; no actor state or turn context is retained.
            pub struct #group { #(#action_fields,)* }
            impl #actor {
                pub const fn actions() -> #group { #group { #(#action_values,)* } }
                pub fn events(&self) -> #group { #group { #(#bound_action_values,)* } }
            }
        }
    };
    let view_call = if render {
        let span = component_renderers
            .first()
            .map_or_else(proc_macro2::Span::call_site, |(_, output, _)| {
                syn::spanned::Spanned::span(output)
            });
        quote::quote_spanned!(span=> ::blockui::render_bound(&Self::new(), cx))
    } else {
        let span = item
            .items
            .iter()
            .find_map(|member| match member {
                syn::ImplItem::Fn(method) if method.sig.ident == "view" => {
                    Some(syn::spanned::Spanned::span(&method.sig.output))
                }
                _ => None,
            })
            .unwrap_or_else(proc_macro2::Span::call_site);
        quote::quote_spanned!(span=> ::blockui::__view_result(Self::new().view(cx)))
    };
    item.items = std::mem::take(&mut item.items)
        .into_iter()
        .enumerate()
        .filter_map(|(index, member)| (!moved_methods.contains(&index)).then_some(member))
        .collect();
    let bound_descriptors = if render && (!bound_accessors.is_empty() || !bound_methods.is_empty())
    {
        let Type::Path(mut path) = actor.as_ref().clone() else {
            return Err(syn::Error::new(
                actor.span(),
                "render actor must be a named type",
            ));
        };
        let segment = path.path.segments.last_mut().unwrap();
        segment.ident = quote::format_ident!("{}Bound", segment.ident);
        segment.arguments = PathArguments::AngleBracketed(syn::parse_quote!(<'cx, 'host>));
        quote! {
            impl<'cx, 'host: 'cx> #path {
                #(#bound_accessors)*
                #(#bound_methods)*
            }
        }
    } else {
        quote!()
    };
    let mut generated_renderers = Vec::new();
    for (cfg, output, renderer) in component_renderers {
        let Type::Path(mut path) = actor.as_ref().clone() else {
            return Err(syn::Error::new(
                actor.span(),
                "component must be a named type",
            ));
        };
        let segment = path.path.segments.last_mut().unwrap();
        segment.ident = quote::format_ident!("{}Bound", segment.ident);
        segment.arguments = PathArguments::AngleBracketed(syn::parse_quote!(<'cx, 'host>));
        generated_renderers.push(quote! {
            #(#cfg)*
            impl<'cx, 'host: 'cx> ::blockui::Render<#output> for #path { #renderer }
        });
    }
    let implementation_trait = if component && !app_root {
        quote!(::blockui::Component)
    } else {
        quote!(::blockui::Actor)
    };
    Ok(quote! {
        #item
        #bound_descriptors
        #(#generated_renderers)*
        #action_group
        impl #actor { #(#accessors)* }
        impl #actor {
            const __LOCAL_ACTIONS: &'static [::blockui::Action] = &[#(#actions),*];
            const __LOCAL_SCOPES: &'static [::blockui::Scope] = &[#(#scopes),*];
            const __LOCAL_COMPUTEDS: &'static [::blockui::ComputedSpec] = &[#(#computeds),*];
            const __ALL_ACTIONS: [::blockui::Action; ::blockui::__parts_len(Self::__COMPONENT_ACTIONS) + Self::__LOCAL_ACTIONS.len()] = ::blockui::__merge_parts(Self::__COMPONENT_ACTIONS, Self::__LOCAL_ACTIONS, ::blockui::Action::remote("unused"));
            const __ALL_SCOPES: [::blockui::Scope; ::blockui::__parts_len(Self::__COMPONENT_SCOPES) + Self::__LOCAL_SCOPES.len()] = ::blockui::__merge_parts(Self::__COMPONENT_SCOPES, Self::__LOCAL_SCOPES, ::blockui::Scope::new("unused", |_| Err(::blockui::Error::Overflow)));
            const __ALL_COMPUTEDS: [::blockui::ComputedSpec; ::blockui::__parts_len(Self::__COMPONENT_COMPUTEDS) + Self::__LOCAL_COMPUTEDS.len()] = ::blockui::__merge_parts(Self::__COMPONENT_COMPUTEDS, Self::__LOCAL_COMPUTEDS, ::blockui::Computed::<i64>::new("unused", |_| Ok(0)).spec());
        }
        impl #implementation_trait for #actor {
            const STATE: &'static [::blockui::StateSpec] = Self::SCHEMA;
            const ACTIONS: &'static [::blockui::Action] = &Self::__ALL_ACTIONS;
            const SCOPES: &'static [::blockui::Scope] = &Self::__ALL_SCOPES;
            const COMPUTEDS: &'static [::blockui::ComputedSpec] = &Self::__ALL_COMPUTEDS;
            fn migrate(cx: &::blockui::Cx<'_>, previous: u64) -> ::blockui::Result<()> {
                #[allow(unused_mut)]
                let mut migration: Option<fn(&::blockui::Cx<'_>, u64) -> ::blockui::Result<()>> = None;
                #(#migration)*
                ::blockui::__migrate_actor(cx, previous,
                    Self::__initialize_record_values_with_scopes,
                    Self::__validate_record_codecs_with_scopes,
                    Self::__migrate_components,
                    Self::__validate_route_histories_with_scopes,
                    migration)
            }
            fn view(cx: &::blockui::Cx<'_>) -> ::blockui::Result<::blockui::View> { #view_call }
        }
    })
}

/// Build a View from an explicit final `builder: &Ui` parameter and node tags.
/// `#[node] expr;` appends a leaf; a marked block starts with its parent expression.
/// The generated wrapper owns the builder and finishes it after construction.
/// The attribute takes no arguments; the builder name comes from the parameter.
/// The body is kept as tokens so incomplete Rust remains available to tooling.
#[proc_macro_attribute]
pub fn view(attributes: TokenStream, input: TokenStream) -> TokenStream {
    sdk_path::resolve(view_tokens(attributes, input))
}
fn view_tokens(attributes: TokenStream, input: TokenStream) -> TokenStream {
    match view_blocks::expand(attributes.into(), input.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

/// Build a UI root with expression markers and optional child blocks.
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    sdk_path::resolve(ui_tokens(input))
}
fn ui_tokens(input: TokenStream) -> TokenStream {
    view_blocks::expand_ui(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Experimental XML-style UI backed by rstml recovery.
#[cfg(feature = "rsx")]
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    sdk_path::resolve(rsx_tokens(input))
}
#[cfg(feature = "rsx")]
fn rsx_tokens(input: TokenStream) -> TokenStream {
    view_blocks::expand_rsx(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(test)]
mod actor_impl_tests {
    #[test]
    fn malformed_scope_preserves_other_generated_accessors() {
        let mut item: syn::ItemImpl = syn::parse_quote! {
            impl Model {
                fn view(&self, cx: &Cx<'_>) -> View { todo!() }
                #[scope]
                fn working(&self, cx: &Cx<'_>) -> View { todo!() }
                #[scope]
                fn broken(&self) -> View { todo!() }
            }
        };
        let original = item.clone();
        let error = super::expand_actor_impl_mode(&mut item, false).unwrap_err();
        let expanded = super::recover_actor_impl(&original, false, error).to_string();
        assert!(expanded.contains("compile_error !"));
        assert!(expanded.contains("scope requires synchronous"));
        assert!(expanded.contains("fn working_scope"));
        assert!(expanded.contains("impl :: blockui :: Actor for Model"));
        assert!(!expanded.contains("fn broken_scope"));
    }
    #[test]
    fn malformed_computed_and_action_preserve_other_accessors() {
        for role in ["computed", "action"] {
            let original: syn::ItemImpl = syn::parse_str(&format!(
                "impl Model {{ fn view(&self, cx: &Cx<'_>) -> View {{ todo!() }}
                 #[{role}] fn broken(&self) -> i64 {{ 1 }}
                 #[scope] fn working(&self, cx: &Cx<'_>) -> View {{ todo!() }} }}"
            ))
            .unwrap();
            let error = super::expand_actor_impl_mode(&mut original.clone(), false).unwrap_err();
            let expanded = super::recover_actor_impl(&original, false, error).to_string();
            assert!(expanded.contains("compile_error !"), "{role}");
            assert!(expanded.contains("fn working_scope"), "{role}");
            assert!(
                expanded.contains("impl :: blockui :: Actor for Model"),
                "{role}"
            );
            assert!(!expanded.contains("fn broken"), "{role}");
        }
    }

    #[test]
    fn independent_method_errors_recover_together_but_global_errors_do_not() {
        let original: syn::ItemImpl = syn::parse_quote! {
            impl Model {
                fn view(&self, cx: &Cx<'_>) -> View { todo!() }
                #[scope] fn working(&self, cx: &Cx<'_>) -> View { todo!() }
                #[scope] fn unfinished_scope(&self) -> View { todo!() }
                #[computed] fn unfinished_computed(&self) -> i64 { 1 }
                #[action] fn unfinished_action(&self) -> Result<()> { Ok(()) }
            }
        };
        let error = super::expand_actor_impl_mode(&mut original.clone(), false).unwrap_err();
        let expanded = super::recover_actor_impl(&original, false, error).to_string();
        assert_eq!(expanded.matches("compile_error !").count(), 3);
        assert!(expanded.contains("fn working_scope"));
        assert!(expanded.contains("impl :: blockui :: Actor for Model"));
        assert!(!expanded.contains("fn unfinished"));
        let mut missing_view = original;
        missing_view.items.remove(0);
        let error = super::expand_actor_impl_mode(&mut missing_view.clone(), false).unwrap_err();
        let expanded = super::recover_actor_impl(&missing_view, false, error).to_string();
        assert!(!expanded.contains("impl :: blockui :: Actor"));
        assert!(!expanded.contains("fn working_scope"));
    }

    #[test]
    fn independent_recovery_keeps_interleaved_ordinary_items() {
        let original: syn::ItemImpl = syn::parse_quote! {
            impl Model {
                const BEFORE: i64 = 1;
                #[scope] fn working(&self, _: &Cx<'_>) -> View { todo!() }
                fn helper(&self) -> i64 { Self::BEFORE }
                #[computed] fn broken(&self) -> i64 { 1 }
                const AFTER: i64 = 2;
                #[action] fn save(&self, _: &Cx<'_>) -> Result<()> { Ok(()) }
                fn view(&self, _: &Cx<'_>) -> View { todo!() }
            }
        };
        let error = super::expand_actor_impl_mode(&mut original.clone(), false).unwrap_err();
        let expanded = super::recover_actor_impl(&original, false, error).to_string();
        assert_eq!(expanded.matches("compile_error !").count(), 1);
        assert!(expanded.contains("computed requires fn method"));
        assert!(expanded.contains("fn working_scope"));
        assert!(expanded.contains("fn save_action"));
        assert!(!expanded.contains("fn broken"));
        let positions: Vec<_> = [
            "const BEFORE",
            "fn working",
            "fn helper",
            "const AFTER",
            "fn save",
            "fn view",
        ]
        .iter()
        .map(|needle| expanded.find(needle).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn cross_method_scope_conflict_still_recovers_unrelated_accessors() {
        let original: syn::ItemImpl = syn::parse_quote! {
            impl Model {
                fn view(&self, _: &Cx<'_>) -> View { todo!() }
                #[scope] fn working(&self, _: &Cx<'_>) -> View { todo!() }
                #[scope(name = "shared")] fn first(&self, _: &Cx<'_>) -> View { todo!() }
                #[scope(name = "shared")] fn second(&self, _: &Cx<'_>) -> View { todo!() }
            }
        };
        let error = super::expand_actor_impl_mode(&mut original.clone(), false).unwrap_err();
        let expanded = super::recover_actor_impl(&original, false, error).to_string();
        assert_eq!(expanded.matches("compile_error !").count(), 2);
        assert!(expanded.contains("duplicate scope name `shared`"));
        assert!(expanded.contains("first scope with this name is here"));
        assert!(expanded.contains("fn working_scope"));
        assert!(expanded.contains("fn second_scope"));
        assert!(!expanded.contains("fn first_scope"));
        assert!(expanded.contains("impl :: blockui :: Actor for Model"));
    }

    #[test]
    fn component_view_requires_a_synchronous_bound_receiver() {
        for method in [
            "async fn view(&self) -> View { todo!() }",
            "fn view(self) -> View { todo!() }",
            "fn view(&mut self) -> View { todo!() }",
            "fn view(&self, cx: &Cx<'_>) -> View { todo!() }",
            "unsafe fn view(&self) -> View { todo!() }",
            "fn view<T>(&self) -> View { todo!() }",
        ] {
            let mut item = syn::parse_str(&format!("impl Counter {{ {method} }}")).unwrap();
            let error = super::expand_implementation_mode(&mut item, true, true).unwrap_err();
            assert!(
                error.to_string().contains("component view requires"),
                "{error}"
            );
        }
    }
    #[test]
    fn component_view_diagnostic_preserves_the_explicit_builder_parameter() {
        for attribute in [quote!(#[view]), quote!(#[blockui::view])] {
            let mut item = syn::parse2(quote! {
                impl Counter {
                    #attribute
                    async fn view(&self, ui: &Ui) -> Result<View> { todo!() }
                }
            })
            .unwrap();
            let error = super::expand_implementation_mode(&mut item, true, true).unwrap_err();
            assert!(
                error.to_string().contains(
                    "component view with #[view] requires synchronous fn view(&self, ui: &Ui)"
                ),
                "{error}"
            );
        }
    }
    #[test]
    fn actor_view_diagnostic_preserves_the_explicit_builder_parameter() {
        for attribute in [quote!(#[view]), quote!(#[blockui::view])] {
            let mut item = syn::parse2(quote! {
                impl Counter {
                    #attribute
                    async fn view(&self, cx: &Cx<'_>, ui: &Ui) -> Result<View> { todo!() }
                }
            })
            .unwrap();
            let error = super::expand_actor_impl_mode(&mut item, false).unwrap_err();
            assert!(
                error.to_string().contains(
                    "view with #[view] requires synchronous fn view(&self, cx: &Cx<'_>, ui: &Ui)"
                ),
                "{error}"
            );
        }
    }
    #[test]
    fn generated_accessors_report_authored_name_collisions() {
        for method in [
            quote!(
                #[action]
                fn save(&self, _: &Cx<'_>) -> Result<()> {
                    Ok(())
                }
            ),
            quote!(
                #[computed]
                fn save(&self, _: &Cx<'_>) -> Result<i64> {
                    Ok(0)
                }
            ),
            quote!(
                #[scope]
                fn save(&self, _: &Cx<'_>) -> View {
                    todo!()
                }
            ),
        ] {
            let accessor = if method.to_string().contains("action") {
                "save_action"
            } else if method.to_string().contains("computed") {
                "save_computed"
            } else {
                "save_scope"
            };
            let name = syn::Ident::new(accessor, proc_macro2::Span::call_site());
            let mut item = syn::parse2(quote! { impl Model {
                #method
                fn #name(&self) {}
            }})
            .unwrap();
            let error = super::expand_actor_impl_mode(&mut item, true).unwrap_err();
            assert!(error.to_string().contains(accessor), "{error}");
            assert!(
                error
                    .into_compile_error()
                    .to_string()
                    .contains("conflicting associated item is declared here")
            );
        }
        let mut conditional = syn::parse_quote! { impl Model {
            #[action] fn save(&self, _: &Cx<'_>) -> Result<()> { Ok(()) }
            #[cfg(any())] fn save_action(&self) {}
        }};
        assert!(super::expand_actor_impl_mode(&mut conditional, true).is_ok());
    }
    #[test]
    fn bound_scopes_require_render_and_synchronous_shared_receiver() {
        for method in [
            "async fn panel(&self) -> View { todo!() }",
            "fn panel(&mut self) -> View { todo!() }",
            "fn panel(&self, cx: &Cx<'_>) -> View { todo!() }",
        ] {
            let mut item: syn::ItemImpl =
                syn::parse_str(&format!("impl Model {{ #[scope(bound)] {method} }}")).unwrap();
            assert!(super::expand_actor_impl_mode(&mut item, true).is_err());
        }
        let mut item: syn::ItemImpl = syn::parse_quote! {
            impl Model { #[scope(bound)] fn panel(&self) -> View { todo!() } }
        };
        assert!(
            super::expand_actor_impl_mode(&mut item, false)
                .unwrap_err()
                .to_string()
                .contains("require actor_impl(render)")
        );
    }
    #[test]
    fn scope_signature_diagnostics_include_the_explicit_ui_builder() {
        for (attributes, signature, expected) in [
            (
                quote!(#[scope]),
                quote!(fn panel(&self)),
                "use #[scope(bound)] for a synchronous method(&self)",
            ),
            (
                quote!(#[scope] #[view]),
                quote!(fn panel(&self, ui: &Ui)),
                "use #[scope(bound)] for a synchronous method(&self, ui: &Ui)",
            ),
            (
                quote!(#[scope]),
                quote!(async fn panel(&self, cx: &Cx<'_>)),
                "scope requires synchronous fn method(&self, cx: &Cx<'_>)",
            ),
            (
                quote!(#[scope(bound)]),
                quote!(async fn panel(&self)),
                "bound scope requires synchronous fn method(&self)",
            ),
            (
                quote!(#[scope] #[view]),
                quote!(async fn panel(&self, cx: &Cx<'_>, ui: &Ui)),
                "scope with #[view] requires synchronous fn method(&self, cx: &Cx<'_>, ui: &Ui)",
            ),
            (
                quote!(#[scope(bound)] #[view]),
                quote!(async fn panel(&self, ui: &Ui)),
                "bound scope with #[view] requires synchronous fn method(&self, ui: &Ui)",
            ),
        ] {
            let mut item = syn::parse2(quote! {
                impl Model { #attributes #signature -> Result<View> { todo!() } }
            })
            .unwrap();
            let error = super::expand_actor_impl_mode(&mut item, true).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
            if !attributes.to_string().contains("bound") {
                assert!(error.to_string().contains("use #[scope(bound)]"), "{error}");
            }
        }
    }

    #[test]
    fn computed_result_accepts_an_explicit_error_type_and_rejects_invalid_arguments() {
        for output in [
            quote!(Result<i64>),
            quote!(std::result::Result<i64, blockui::Error>),
        ] {
            let mut item = syn::parse2(quote! {
                impl Model {
                    #[computed] fn value(&self, cx: &Cx<'_>) -> #output { todo!() }
                    fn view(&self, _: &Cx<'_>) -> View { todo!() }
                }
            })
            .unwrap();
            assert!(expand_actor_impl_mode(&mut item, false).is_ok());
        }
        for output in [
            quote!(Result),
            quote!(Result<i64, Error, Error>),
            quote!(Result<i64, 3>),
        ] {
            let mut item = syn::parse2(quote! {
                impl Model {
                    #[computed] fn value(&self, cx: &Cx<'_>) -> #output { todo!() }
                    fn view(&self, _: &Cx<'_>) -> View { todo!() }
                }
            })
            .unwrap();
            assert!(
                expand_actor_impl_mode(&mut item, false)
                    .unwrap_err()
                    .to_string()
                    .contains("computed Result requires a value type and optional error type")
            );
        }
    }
    #[test]
    fn missing_bound_computed_explains_the_render_receiver() {
        for render in [false, true] {
            let mut item = syn::parse_quote! {
                impl Model { #[computed] fn double(&self) -> i64 { 2 } }
            };
            let error = super::expand_actor_impl_mode(&mut item, render).unwrap_err();
            assert!(error.to_string().contains("computed requires fn method"));
            assert_eq!(error.to_string().contains("use #[computed(bound)]"), render);
        }
    }

    #[test]
    fn bound_computeds_require_render_and_no_context_or_payload() {
        for method in [
            "async fn double(&self) -> Result<i64> { todo!() }",
            "fn double(&mut self) -> Result<i64> { todo!() }",
            "fn double(&self, cx: &Cx<'_>) -> Result<i64> { todo!() }",
        ] {
            let mut item: syn::ItemImpl =
                syn::parse_str(&format!("impl Model {{ #[computed(bound)] {method} }}")).unwrap();
            assert!(super::expand_actor_impl_mode(&mut item, true).is_err());
        }
        let mut item: syn::ItemImpl = syn::parse_quote! {
            impl Model { #[computed(bound)] fn double(&self) -> Result<i64> { Ok(2) } }
        };
        assert!(
            super::expand_actor_impl_mode(&mut item, false)
                .unwrap_err()
                .to_string()
                .contains("requires actor_impl(render)")
        );
    }
    #[test]
    fn bound_actions_require_render_and_synchronous_shared_receiver() {
        for method in [
            "async fn change(&self) -> Result<()> { todo!() }",
            "fn change(&mut self) -> Result<()> { todo!() }",
            "fn change(&self, a: i64, b: i64) -> Result<()> { todo!() }",
            "fn change(&self, value: Vec<String>) -> Result<()> { todo!() }",
        ] {
            let mut item: syn::ItemImpl =
                syn::parse_str(&format!("impl Model {{ #[action(bound)] {method} }}")).unwrap();
            assert!(super::expand_actor_impl_mode(&mut item, true).is_err());
        }
        let mut item: syn::ItemImpl = syn::parse_quote! {
            impl Model { #[action(bound)] fn change(&self) -> Result<()> { Ok(()) } }
        };
        assert!(
            super::expand_actor_impl_mode(&mut item, false)
                .unwrap_err()
                .to_string()
                .contains("require actor_impl(render)")
        );
    }

    #[test]
    fn record_fields_are_inferred_and_keep_existing_annotation_validation() {
        let mut inferred: syn::ItemStruct = syn::parse_quote! {
            struct Model {
                #[state(default = Document::new("Draft"))]
                document: State<domain::Document>,
            }
        };
        let generated = expand(&mut inferred, "model").unwrap().to_string();
        assert!(generated.contains("Record > :: FORMAT"));
        assert!(generated.contains("__initialize"));
        for declaration in [
            quote::quote! { struct Model { #[state(record_value, record_value)] document: State<Document> } },
            quote::quote! { struct Model { #[state(codec = Document)] document: State<Document> } },
            quote::quote! { struct Model { document: Object<Document> } },
        ] {
            let mut item: syn::ItemStruct = syn::parse2(declaration).unwrap();
            assert!(expand(&mut item, "model").is_err());
        }
    }

    #[test]
    fn record_codecs_reject_wrong_storage_and_duplicate_annotations() {
        for (declaration, message) in [
            (
                quote::quote! { struct Model { #[state(codec = Document)] value: State<i64> } },
                "codec requires State<Vec<String>>",
            ),
            (
                quote::quote! { struct Model { #[state(codec = Document, codec = Document)] value: State<Vec<String>> } },
                "expected a unique key, default or codec",
            ),
            (
                quote::quote! { struct Model { #[state(route = Page)] value: State<i64> } },
                "route requires State<Vec<String>>",
            ),
            (
                quote::quote! { struct Model { #[state(route = Page, route = Page)] value: State<Vec<String>> } },
                "expected a unique key, default or codec",
            ),
            (
                quote::quote! { struct Model { #[state(route = Page, codec = Document)] value: State<Vec<String>> } },
                "expected a unique key, default or codec",
            ),
            (
                quote::quote! { struct Model { #[state(route_request = Page)] value: State<Vec<String>> } },
                "route_request requires State<String>",
            ),
            (
                quote::quote! { struct Model { #[state(route_request = Page, route = Page)] value: State<String> } },
                "expected a unique key, default or codec",
            ),
        ] {
            let mut item: syn::ItemStruct = syn::parse2(declaration).unwrap();
            assert!(
                expand(&mut item, "model")
                    .unwrap_err()
                    .to_string()
                    .contains(message)
            );
        }
    }
    #[test]
    fn receiver_keyed_text_actions_register_one_typed_handler_and_reject_replies() {
        for ty in ["KeyedText", "blockui::KeyedText"] {
            let mut input: syn::ItemImpl = syn::parse_str(&format!("impl Demo {{ #[action] fn edit(&self, cx: &Cx<'_>, edit: {ty}) -> Result<()> {{ todo!() }} }}")).unwrap();
            let output = expand_actor_impl_mode(&mut input, true)
                .unwrap()
                .to_string();
            assert!(output.contains("KeyedTextAction"));
            assert!(output.contains("edit_action"));
        }
        for signature in [
            "fn edit(&self, cx: &Cx<'_>, edit: KeyedText) -> Result<i64>",
            "fn edit(&self, cx: &Cx<'_>, edit: KeyedText) -> Result<String>",
            "fn edit(&self, cx: &Cx<'_>, edit: other::KeyedText) -> Result<()>",
        ] {
            let mut input: syn::ItemImpl = syn::parse_str(&format!(
                "impl Demo {{ #[action] {signature} {{ todo!() }} }}"
            ))
            .unwrap();
            assert!(expand_actor_impl_mode(&mut input, true).is_err());
        }
    }
    #[test]
    fn receiver_point_actions_are_typed_and_reject_scalar_replies() {
        for ty in ["LocalPoint", "blockui::LocalPoint"] {
            let mut input: syn::ItemImpl = syn::parse_str(&format!("impl Demo {{ #[action] fn draw(&self, cx: &Cx<'_>, point: {ty}) -> Result<()> {{ todo!() }} }}")).unwrap();
            let output = expand_actor_impl_mode(&mut input, true)
                .unwrap()
                .to_string();
            assert!(output.contains("PointAction"));
        }
        for signature in [
            "fn draw(&self, cx: &Cx<'_>, point: LocalPoint) -> Result<i64>",
            "fn draw(&self, cx: &Cx<'_>, point: other::LocalPoint) -> Result<()>",
        ] {
            let mut input: syn::ItemImpl = syn::parse_str(&format!(
                "impl Demo {{ #[action] {signature} {{ todo!() }} }}"
            ))
            .unwrap();
            assert!(expand_actor_impl_mode(&mut input, true).is_err());
        }
    }
    #[test]
    fn host_field_identity_options_reject_empty_duplicate_and_unknown_values() {
        for field in [
            quote!(#[task(key = "")] timer: Task),
            quote!(#[task(key = "one", key = "two")] timer: Task),
            quote!(#[resource(reload = ReloadPolicy::Cancel, reload = ReloadPolicy::Preserve)] file: Resource<File>),
            quote!(#[resource(unknown = true)] file: Resource<File>),
        ] {
            let mut item: syn::ItemStruct = syn::parse2(quote!(struct Model { #field })).unwrap();
            assert!(expand(&mut item, "model").is_err());
        }
    }
    #[test]
    fn host_attributes_report_the_wrong_field_at_the_declaration() {
        for (field, expected) in [
            (
                quote!(#[task(key = "wrong")] count: State<i64>),
                "task attributes require a Task field",
            ),
            (
                quote!(#[resource(key = "wrong")] timer: Task),
                "resource attributes require a Resource<File> or Resource<Socket> field",
            ),
            (
                quote!(#[task(key = "wrong")] file: Resource<File>),
                "task attributes require a Task field",
            ),
            (
                quote!(#[resource(key = "wrong")] count: State<i64>),
                "resource attributes require a Resource<File> or Resource<Socket> field",
            ),
        ] {
            let mut item = syn::parse2(quote!(struct Model { #field })).unwrap();
            assert_eq!(
                expand(&mut item, "model").unwrap_err().to_string(),
                expected
            );
        }
    }
    #[test]
    fn host_fields_reject_duplicate_unconditional_identities() {
        for fields in [
            quote!(#[task(key = "shared")] first: Task, #[task(key = "shared")] second: Task),
            quote!(#[resource(key = "shared")] first: Resource<File>, #[resource(key = "shared")] second: Resource<File>),
            quote!(first: Task, #[task(key = "model.first")] second: Task),
        ] {
            let mut item = syn::parse2(quote!(struct Model { #fields })).unwrap();
            assert!(
                expand(&mut item, "model")
                    .unwrap_err()
                    .to_string()
                    .contains("duplicate")
            );
        }
        // Tasks and files occupy distinct host registries; sharing spelling is valid.
        let mut item = syn::parse2(quote!(
            struct Model {
                #[task(key = "shared")]
                task: Task,
                #[resource(key = "shared")]
                file: Resource<File>,
            }
        ))
        .unwrap();
        assert!(expand(&mut item, "model").is_ok());
        // Mutually exclusive conditional declarations must remain possible.
        let mut item = syn::parse2(quote!(
            struct Model {
                #[cfg(feature = "one")]
                #[task(key = "shared")]
                first: Task,
                #[cfg(not(feature = "one"))]
                #[task(key = "shared")]
                second: Task,
            }
        ))
        .unwrap();
        assert!(expand(&mut item, "model").is_ok());
    }
    #[test]
    fn file_resource_fields_exclude_native_handles_from_state() {
        let mut valid: syn::ItemStruct = syn::parse2(quote! {
            struct Model { document: ::blockui::Resource<::blockui::File> }
        })
        .unwrap();
        assert!(expand(&mut valid, "model").is_ok());
        let mut invalid: syn::ItemStruct = syn::parse2(quote! {
            struct Model { #[state(default = 0)] document: Resource<File> }
        })
        .unwrap();
        assert!(
            expand(&mut invalid, "model")
                .unwrap_err()
                .to_string()
                .contains("not durable state slots")
        );
    }
    #[test]
    fn task_fields_accept_descriptors_but_reject_durable_state_attributes() {
        let mut valid: syn::ItemStruct = syn::parse2(quote! {
            struct Model { count: State<i64>, timer: ::blockui::Task }
        })
        .unwrap();
        assert!(expand(&mut valid, "model").is_ok());
        let mut invalid: syn::ItemStruct = syn::parse2(quote! {
            struct Model { #[state(default = 0)] timer: Task }
        })
        .unwrap();
        assert!(
            expand(&mut invalid, "model")
                .unwrap_err()
                .to_string()
                .contains("not durable state slots")
        );
    }
    #[test]
    fn text_collection_declarations_accept_empty_and_explicit_defaults() {
        for ty in [
            quote!(Vec<String>),
            quote!(std::vec::Vec<std::string::String>),
        ] {
            let mut item: syn::ItemStruct = syn::parse2(quote! {
                struct Model { items: State<#ty> }
            })
            .unwrap();
            assert!(expand(&mut item, "model").is_ok());
        }
        let mut item: syn::ItemStruct = syn::parse2(quote! {
            struct Model { #[state(default = &["initial"])] items: State<Vec<String>> }
        })
        .unwrap();
        assert!(expand(&mut item, "model").is_ok());
    }
    #[test]
    fn invalid_scope_declarations_fail_at_the_method() {
        for method in [
            quote!(
                #[scope]
                async fn panel(&self, _: &Cx<'_>) -> View {
                    todo!()
                }
            ),
            quote!(
                #[scope]
                fn panel(&mut self, _: &Cx<'_>) -> View {
                    todo!()
                }
            ),
            quote!(
                #[scope]
                fn panel(&self) -> View {
                    todo!()
                }
            ),
            quote!(
                #[scope(name = "")]
                fn panel(&self, _: &Cx<'_>) -> View {
                    todo!()
                }
            ),
            quote!(
                #[scope(name = "a", name = "b")]
                fn panel(&self, _: &Cx<'_>) -> View {
                    todo!()
                }
            ),
            quote!(
                #[scope = "a"]
                fn panel(&self, _: &Cx<'_>) -> View {
                    todo!()
                }
            ),
        ] {
            let mut item = syn::parse2(quote!(impl Model { #method })).unwrap();
            assert!(expand_actor_impl_mode(&mut item, true).is_err());
        }
        let mut duplicate = syn::parse2(quote! { impl Model {
            #[scope(name = "same")] fn first(&self, _: &Cx<'_>) -> View { todo!() }
            #[scope(name = "same")] fn second(&self, _: &Cx<'_>) -> View { todo!() }
        } })
        .unwrap();
        let error = expand_actor_impl_mode(&mut duplicate, true).unwrap_err();
        assert!(error.to_string().contains("duplicate scope name `same`"));
        assert!(
            error
                .into_compile_error()
                .to_string()
                .contains("first scope with this name is here")
        );
    }
    #[test]
    fn scope_missing_return_type_reports_the_authored_signature() {
        for method in [
            quote!(
                #[scope]
                fn panel(&self, _: &Cx<'_>) {}
            ),
            quote!(
                #[scope(bound)]
                fn panel(&self) {}
            ),
        ] {
            let mut item = syn::parse2(quote!(impl Model { #method })).unwrap();
            let error = expand_actor_impl_mode(&mut item, true).unwrap_err();
            assert_eq!(
                error.to_string(),
                "scope requires an explicit View or Result<View> return type"
            );
        }
    }
    #[test]
    fn scope_accessors_preserve_authored_docs_and_explain_tracking() {
        for bound in [false, true] {
            let method = if bound {
                quote!(
                    #[scope(bound)]
                    fn panel(&self) -> View {
                        todo!()
                    }
                )
            } else {
                quote!(
                    #[scope]
                    fn panel(&self, _: &Cx<'_>) -> View {
                        todo!()
                    }
                )
            };
            let mut item = syn::parse2(quote! { impl Model {
                /// Show the active workspace.
                #method
            } })
            .unwrap();
            let expanded = expand_actor_impl_mode(&mut item, true).unwrap().to_string();
            assert_eq!(expanded.matches("Show the active workspace.").count(), 3);
            assert_eq!(
                expanded
                    .matches("State reads are tracked dynamically")
                    .count(),
                2
            );
            assert_eq!(
                expanded
                    .matches("obtaining this descriptor does not run the builder")
                    .count(),
                2
            );
        }
    }
    #[test]
    fn actor_namespace_never_silently_replaces_an_explicit_name() {
        assert_eq!(actor_namespace(quote!(), "Model".into()).unwrap(), "Model");
        assert_eq!(
            actor_namespace(quote!(name = "stable.model"), "Model".into()).unwrap(),
            "stable.model"
        );
        assert!(
            actor_namespace(quote!(name = "first", name = "second"), "Model".into())
                .unwrap_err()
                .to_string()
                .contains("only be declared once")
        );
    }

    #[test]
    fn invalid_view_signatures_fail_at_the_authored_method() {
        for signature in [
            quote!(async fn view(&self, _: &Cx<'_>)),
            quote!(unsafe fn view(&self, _: &Cx<'_>)),
            quote!(fn view<T>(&self, _: &Cx<'_>)),
            quote!(fn view(&mut self, _: &Cx<'_>)),
            quote!(fn view(self, _: &Cx<'_>)),
            quote!(fn view(_: &Cx<'_>)),
            quote!(fn view(&self)),
        ] {
            let mut item: syn::ItemImpl = syn::parse2(quote! {
                impl Model { #signature -> View { todo!() } }
            })
            .unwrap();
            assert!(
                expand_actor_impl_mode(&mut item, false)
                    .unwrap_err()
                    .to_string()
                    .contains("view requires")
            );
        }
    }

    #[test]
    fn invalid_migration_signatures_fail_at_the_authored_method() {
        for signature in [
            quote!(async fn migrate(&self, _: &Cx<'_>, _: u64)),
            quote!(unsafe fn migrate(&self, _: &Cx<'_>, _: u64)),
            quote!(fn migrate<T>(&self, _: &Cx<'_>, _: u64)),
            quote!(fn migrate(&mut self, _: &Cx<'_>, _: u64)),
            quote!(fn migrate(self, _: &Cx<'_>, _: u64)),
            quote!(fn migrate(_: &Cx<'_>, _: u64)),
            quote!(fn migrate(&self, _: &Cx<'_>)),
        ] {
            let mut item: syn::ItemImpl = syn::parse2(quote! {
                impl Model { #signature -> Result<()> { Ok(()) } }
            })
            .unwrap();
            assert!(
                expand_actor_impl_mode(&mut item, true)
                    .unwrap_err()
                    .to_string()
                    .contains("migration requires")
            );
        }
    }

    #[test]
    fn lifecycle_methods_cannot_be_registered_as_event_or_computation() {
        for method in ["view", "migrate"] {
            for role in ["action", "computed", "scope"] {
                let method = syn::Ident::new(method, proc_macro2::Span::call_site());
                let role = syn::Ident::new(role, proc_macro2::Span::call_site());
                let mut item: syn::ItemImpl = syn::parse2(quote! {
                    impl Model {
                        #[#role]
                        fn #method(&self, _: &Cx<'_>) -> Result<i64> { Ok(1) }
                    }
                })
                .unwrap();
                assert!(
                    expand_actor_impl_mode(&mut item, true)
                        .unwrap_err()
                        .to_string()
                        .contains("lifecycle methods")
                );
            }
        }
    }

    #[test]
    fn rejects_multiple_method_roles_at_the_authored_attribute() {
        for attrs in [
            quote!(#[action] #[computed]),
            quote!(#[computed] #[action]),
            quote!(#[action] #[action]),
            quote!(#[computed] #[computed]),
            quote!(#[scope] #[action]),
            quote!(#[computed] #[scope]),
            quote!(#[scope] #[scope]),
        ] {
            let mut item: syn::ItemImpl = syn::parse2(quote! {
                impl Model {
                    #attrs
                    fn value(&self, _: &Cx<'_>) -> Result<i64> { Ok(1) }
                }
            })
            .unwrap();
            let error = expand_actor_impl_mode(&mut item, true).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("only one action, computed or scope")
            );
            assert!(
                error
                    .into_compile_error()
                    .to_string()
                    .contains("first method role is declared here; keep only one role annotation")
            );
        }
    }

    #[test]
    fn explicit_and_generated_computed_names_cannot_collide() {
        for reverse in [false, true] {
            let first: syn::ImplItem = syn::parse_quote! {
                #[computed]
                fn total(&self, _: &Cx<'_>) -> Result<i64> { Ok(1) }
            };
            let second: syn::ImplItem = syn::parse_quote! {
                #[computed(name = "Model.total")]
                fn other(&self, _: &Cx<'_>) -> Result<bool> { Ok(true) }
            };
            let mut item: syn::ItemImpl = syn::parse_quote!(impl Model {});
            item.items = if reverse {
                vec![second, first]
            } else {
                vec![first, second]
            };
            assert!(
                expand_actor_impl_mode(&mut item, true)
                    .unwrap_err()
                    .to_string()
                    .contains("computed name")
            );
        }
    }

    #[test]
    fn explicit_and_generated_action_names_cannot_collide() {
        for reverse in [false, true] {
            let first: syn::ImplItem = syn::parse_quote! {
                #[action]
                fn save(&self, _: &Cx<'_>) -> Result<()> { Ok(()) }
            };
            let second: syn::ImplItem = syn::parse_quote! {
                #[action(name = "Model.save")]
                fn other(&self, _: &Cx<'_>) -> Result<()> { Ok(()) }
            };
            let mut item: syn::ItemImpl = syn::parse_quote!(impl Model {});
            item.items = if reverse {
                vec![second, first]
            } else {
                vec![first, second]
            };
            assert!(
                expand_actor_impl_mode(&mut item, true)
                    .unwrap_err()
                    .to_string()
                    .contains("action name")
            );
        }
    }

    #[test]
    fn duplicate_explicit_computed_names_fail_before_export() {
        let mut item: syn::ItemImpl = syn::parse_quote! {
            impl Model {
                #[computed(name = "total")]
                fn first(&self, _: &Cx<'_>) -> Result<i64> { Ok(1) }
                #[computed(name = "total")]
                fn second(&self, _: &Cx<'_>) -> Result<bool> { Ok(true) }
            }
        };
        let error = expand_actor_impl_mode(&mut item, true).unwrap_err();
        let messages: Vec<_> = error.into_iter().map(|error| error.to_string()).collect();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("duplicate explicit computed name `total`"));
        assert!(messages[1].contains("first computed with this name is here"));
    }
    #[test]
    fn conditional_computed_names_can_share_an_identity() {
        let mut item: syn::ItemImpl = syn::parse_quote! {
            impl Model {
                #[computed(name = "total")]
                #[cfg(feature = "one")]
                fn first(&self, _: &Cx<'_>) -> Result<i64> { Ok(1) }
                #[computed(name = "total")]
                #[cfg_attr(feature = "one", cfg(any()))]
                fn second(&self, _: &Cx<'_>) -> Result<i64> { Ok(2) }
            }
        };
        assert!(expand_actor_impl_mode(&mut item, true).is_ok());
    }

    #[test]
    fn duplicate_explicit_action_names_fail_before_export() {
        let mut item: syn::ItemImpl = syn::parse_quote! {
            impl Model {
                #[action(name = "save")]
                fn first(&self, _: &Cx<'_>) -> Result<()> { Ok(()) }
                #[action(name = "save")]
                fn second(&self, _: &Cx<'_>, value: bool) -> Result<()> { Ok(()) }
            }
        };
        let error = expand_actor_impl_mode(&mut item, true).unwrap_err();
        let messages: Vec<_> = error.into_iter().map(|error| error.to_string()).collect();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("duplicate explicit action name `save`"));
        assert!(messages[1].contains("first action with this name is here"));
    }
    #[test]
    fn conditional_action_names_can_share_an_identity() {
        let mut item: syn::ItemImpl = syn::parse_quote! {
            impl Model {
                #[action(name = "save")]
                #[cfg(feature = "one")]
                fn first(&self, _: &Cx<'_>) -> Result<()> { Ok(()) }
                #[action(name = "save")]
                #[cfg_attr(feature = "one", cfg(any()))]
                fn second(&self, _: &Cx<'_>) -> Result<()> { Ok(()) }
            }
        };
        assert!(expand_actor_impl_mode(&mut item, true).is_ok());
    }

    #[test]
    fn duplicate_unconditional_state_keys_fail_at_the_declaration() {
        let mut item: ItemStruct = syn::parse_quote! {
            struct Model {
                #[state(key = "shared")]
                first: State<i64>,
                #[state(key = "shared")]
                second: State<bool>,
            }
        };
        let error = expand(&mut item, "model").unwrap_err();
        let messages: Vec<_> = error.into_iter().map(|error| error.to_string()).collect();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("duplicate durable state key `shared`"));
        assert!(messages[1].contains("first field with this state key is here"));
    }
    #[test]
    fn conditional_state_keys_defer_to_active_schema_validation() {
        let mut item: ItemStruct = syn::parse_quote! {
            struct Model {
                #[state(key = "shared")]
                #[cfg(feature = "one")]
                first: State<i64>,
                #[state(key = "shared")]
                #[cfg_attr(feature = "one", cfg(any()))]
                second: State<i64>,
            }
        };
        assert!(expand(&mut item, "model").is_ok());
    }

    use super::*;
    #[test]
    fn render_mode_rejects_two_view_boundaries() {
        let mut empty: syn::ItemImpl = syn::parse_quote!(impl Counter {});
        assert!(expand_actor_impl_mode(&mut empty, true).is_ok());
        let mut ambiguous: syn::ItemImpl = syn::parse_quote!(impl Counter {
            fn view(&self, cx: &Cx<'_>) -> Result<View> { todo!() }
        });
        assert!(expand_actor_impl_mode(&mut ambiguous, true).is_err());
    }
    #[test]
    fn rejects_callbacks_that_cannot_use_receiver_adapter() {
        for method in [
            "fn change(cx: &Cx<'_>) -> Result<()> { Ok(()) }",
            "async fn change(&self, cx: &Cx<'_>) -> Result<()> { Ok(()) }",
            "fn change(&mut self, cx: &Cx<'_>) -> Result<()> { Ok(()) }",
            "fn change<T>(&self, cx: &Cx<'_>) -> Result<()> { Ok(()) }",
            "fn change(&self) -> Result<()> { Ok(()) }",
            "fn change(&self, cx: &Cx<'_>, value: &bool) -> Result<()> { Ok(()) }",
            "fn change(&self, cx: &Cx<'_>, value: Alias) -> Result<()> { Ok(()) }",
        ] {
            let mut item: syn::ItemImpl = syn::parse_str(&format!("impl Counter {{ #[action] {method} fn view(&self, cx: &Cx<'_>) -> Result<View> {{ todo!() }} }}")).unwrap();
            assert!(expand_actor_impl(&mut item).is_err(), "accepted {method}");
        }
    }
    #[test]
    fn string_requests_generate_current_receiver_adapter_in_both_modes() {
        for payload in ["String", "std::string::String", "alloc::string::String"] {
            for reply in ["String", "std::string::String", "alloc::string::String"] {
                for render in [false, true] {
                    let view = if render {
                        ""
                    } else {
                        "fn view(&self, cx: &Cx<'_>) -> Result<View> { todo!() }"
                    };
                    let mut item: syn::ItemImpl = syn::parse_str(&format!("impl Client {{ #[action] fn query(&self, cx: &Cx<'_>, text: {payload}) -> Result<{reply}> {{ todo!() }} {view} }}")).unwrap();
                    let tokens = expand_actor_impl_mode(&mut item, render)
                        .unwrap()
                        .to_string();
                    assert!(tokens.contains("TextAction :: request"));
                    assert!(tokens.contains("query_action"));
                    assert!(!tokens.contains("RequestAction"));
                }
            }
        }
        for reply in ["other::String", "String<u8>", "core::primitive::String"] {
            let mut item: syn::ItemImpl = syn::parse_str(&format!("impl Client {{ #[action] fn query(&self, cx: &Cx<'_>, text: String) -> Result<{reply}> {{ todo!() }} fn view(&self, cx: &Cx<'_>) -> Result<View> {{ todo!() }} }}")).unwrap();
            assert!(expand_actor_impl(&mut item).is_err());
        }
    }
    #[test]
    fn string_message_actions_use_text_tokens_and_reject_unsupported_signatures() {
        for payload in ["String", "std::string::String", "alloc::string::String"] {
            for render in [false, true] {
                let view = if render {
                    ""
                } else {
                    "fn view(&self, cx: &Cx<'_>) -> Result<View> { todo!() }"
                };
                let mut item: syn::ItemImpl = syn::parse_str(&format!("impl Client {{ #[action] fn rename(&self, cx: &Cx<'_>, name: {payload}) -> Result<()> {{ todo!() }} {view} }}")).unwrap();
                let tokens = expand_actor_impl_mode(&mut item, render)
                    .unwrap()
                    .to_string();
                assert!(tokens.contains("TextAction"));
                assert!(tokens.contains("spec"));
                assert!(!tokens.contains("RequestAction"));
            }
        }
        for method in [
            "fn rename(&self, cx: &Cx<'_>, name: &str) -> Result<()> { todo!() }",
            "fn rename(&self, cx: &Cx<'_>, name: other::String) -> Result<()> { todo!() }",
            "fn rename(&self, cx: &Cx<'_>, name: core::primitive::String) -> Result<()> { todo!() }",
            "fn rename(&self, cx: &Cx<'_>, name: String<u8>) -> Result<()> { todo!() }",
            "fn rename(&self, cx: &Cx<'_>, name: String) -> Result<i64> { todo!() }",
            "fn rename(&self, cx: &Cx<'_>, name: i64) -> Result<String> { todo!() }",
        ] {
            let mut item: syn::ItemImpl =
                syn::parse_str(&format!("impl Client {{ #[action] {method} }}")).unwrap();
            assert!(expand_actor_impl_mode(&mut item, true).is_err(), "{method}");
        }
    }
    #[test]
    fn request_error_actions_accept_sdk_paths_and_reject_unresolved_aliases() {
        for payload in [
            "RequestError",
            "blockui::RequestError",
            "blockui::abi::RequestError",
            "blockui_abi::RequestError",
        ] {
            let mut item: syn::ItemImpl = syn::parse_str(&format!("impl Client {{ #[action] fn failed(&self, cx: &Cx<'_>, error: {payload}) -> Result<()> {{ todo!() }} }}")).unwrap();
            let tokens = expand_actor_impl_mode(&mut item, true).unwrap().to_string();
            assert!(tokens.contains("request_error"), "{payload}");
        }
        for payload in [
            "Alias",
            "other::RequestError",
            "RequestError<i64>",
            "&RequestError",
        ] {
            let mut item: syn::ItemImpl = syn::parse_str(&format!("impl Client {{ #[action] fn failed(&self, cx: &Cx<'_>, error: {payload}) -> Result<()> {{ todo!() }} }}")).unwrap();
            assert!(
                expand_actor_impl_mode(&mut item, true).is_err(),
                "{payload}"
            );
        }
    }
    #[test]
    fn scalar_reply_actions_accept_mixed_types_and_reject_non_scalar_results() {
        for input in ["i64", "bool", "f64"] {
            for reply in ["i64", "bool", "f64"] {
                let mut item: syn::ItemImpl = syn::parse_str(&format!("impl Queries {{ #[action] fn query(&self, cx: &Cx<'_>, value: {input}) -> Result<{reply}> {{ todo!() }} }}")).unwrap();
                assert!(expand_actor_impl_mode(&mut item, true).is_ok());
            }
        }
        for result in [
            "Result<String>",
            "Result<Vec<i64>>",
            "Result<&str>",
            "Alias",
        ] {
            let mut item: syn::ItemImpl = syn::parse_str(&format!("impl Queries {{ #[action] fn query(&self, cx: &Cx<'_>, value: i64) -> {result} {{ todo!() }} }}")).unwrap();
            assert!(expand_actor_impl_mode(&mut item, true).is_err(), "{result}");
        }
        let mut error: syn::ItemImpl = syn::parse_quote!(impl Queries {
            #[action] fn failed(&self, cx: &Cx<'_>, value: RequestError) -> Result<i64> { todo!() }
        });
        assert!(expand_actor_impl_mode(&mut error, true).is_err());
    }
    #[test]
    fn requires_view_and_nonempty_action_name() {
        for input in [
            "impl Counter { #[action] fn change(&self, cx: &Cx<'_>) -> Result<()> { Ok(()) } }",
            "impl Counter { #[action(name = \"\")] fn change(&self, cx: &Cx<'_>) -> Result<()> { Ok(()) } fn view(&self, cx: &Cx<'_>) -> Result<View> { todo!() } }",
        ] {
            let mut item = syn::parse_str(input).unwrap();
            assert!(expand_actor_impl(&mut item).is_err());
        }
    }
}
