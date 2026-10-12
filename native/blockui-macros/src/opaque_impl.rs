//! Actor adapters inspect method declarations, leaving body expressions to Rust.
use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use syn::parse::{ParseStream, Parser, discouraged::Speculative};

pub fn parse(input: TokenStream) -> syn::Result<syn::ItemImpl> {
    let mut tokens: Vec<_> = input.into_iter().collect();
    let body = match tokens.pop() {
        Some(TokenTree::Group(body)) => body,
        last => {
            tokens.extend(last);
            return syn::parse2(tokens.into_iter().collect());
        }
    };
    if body.delimiter() != Delimiter::Brace {
        tokens.push(TokenTree::Group(body));
        return syn::parse2(tokens.into_iter().collect());
    }
    let mut empty = Group::new(Delimiter::Brace, TokenStream::new());
    empty.set_span(body.span());
    tokens.push(TokenTree::Group(empty));
    let mut item: syn::ItemImpl = syn::parse2(tokens.into_iter().collect())?;
    let (inner_attrs, members) = members.parse2(body.stream())?;
    item.attrs.extend(inner_attrs);
    item.items = members;
    Ok(item)
}

pub fn parse_function(tokens: TokenStream) -> syn::Result<syn::ItemFn> {
    function.parse2(tokens)
}

pub(super) fn function(input: ParseStream<'_>) -> syn::Result<syn::ItemFn> {
    let attrs = input.call(syn::Attribute::parse_outer)?;
    let vis = input.parse()?;
    let sig = input.parse()?;
    let content;
    let brace_token = syn::braced!(content in input);
    let body: TokenStream = content.parse()?;
    Ok(syn::ItemFn {
        attrs,
        vis,
        sig,
        block: Box::new(syn::Block {
            brace_token,
            stmts: vec![syn::Stmt::Expr(syn::Expr::Verbatim(body), None)],
        }),
    })
}

fn members(input: ParseStream<'_>) -> syn::Result<(Vec<syn::Attribute>, Vec<syn::ImplItem>)> {
    let inner_attrs = input.call(syn::Attribute::parse_inner)?;
    let mut items = Vec::new();
    while !input.is_empty() {
        let method = input.fork();
        let attrs = method.call(syn::Attribute::parse_outer)?;
        let vis = method.parse::<syn::Visibility>()?;
        let defaultness = method.parse::<Option<syn::Token![default]>>()?;
        if let Ok(sig) = method.parse::<syn::Signature>()
            && method.peek(syn::token::Brace)
        {
            let content;
            let brace_token = syn::braced!(content in method);
            let body: TokenStream = content.parse()?;
            input.advance_to(&method);
            items.push(syn::ImplItem::Fn(syn::ImplItemFn {
                attrs,
                vis,
                defaultness,
                sig,
                block: syn::Block {
                    brace_token,
                    stmts: vec![syn::Stmt::Expr(syn::Expr::Verbatim(body), None)],
                },
            }));
        } else {
            // Constants, associated types and item macros retain syn validation.
            items.push(input.parse()?);
        }
    }
    Ok((inner_attrs, items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    #[test]
    fn preserves_method_tokens_and_other_members() {
        let source = quote! {
            impl Model {
                #![allow(dead_code)]
                const NAME: &'static str = "model";
                type Value = i64;
                items! { fn injected() {} }
                /// Helper with nested items and const generic expressions.
                pub async fn helper<const N: usize>(&self, value: [u8; { N }]) -> [u8; N] {
                    fn nested() { let _ = 1; }
                    macro_rules! local { () => { 1 }; }
                    value
                }
            }
        };
        let parsed = parse(source.clone()).unwrap();
        assert_eq!(quote!(#parsed).to_string(), source.to_string());
    }
    #[test]
    fn function_bodies_preserve_incomplete_expressions_and_attributes() {
        let source = quote! {
            /// Component documentation.
            #[blockui::view]
            pub fn Panel(count: State<i64>, ui: &Ui) -> View {
                #![allow(unused_variables)]
                #[node] Text::new(count.);
            }
        };
        let function = parse_function(source.clone()).unwrap();
        assert_eq!(quote!(#function).to_string(), source.to_string());
        assert!(syn::parse2::<syn::ItemFn>(source).is_err());
        assert!(parse_function(quote! { fn Panel() {} trailing }).is_err());
        assert!(parse_function(quote! { struct Panel; }).is_err());
    }
    #[test]
    fn incomplete_body_remains_available_for_editor_expansion() {
        let parsed = parse(quote! { impl Model {
            fn view(&self, cx: &Cx<'_>) -> View { self. }
        } })
        .unwrap();
        assert!(quote!(#parsed).to_string().contains("self ."));
        assert!(parse(quote! { impl Model { fn view(&self, : ) {} } }).is_err());
        assert!(parse(quote! { impl Model {} trailing }).is_err());
        assert!(parse(quote! { impl Model {} ; }).is_err());
    }
}
