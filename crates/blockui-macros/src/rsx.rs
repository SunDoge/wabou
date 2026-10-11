//! Optional RSX experiment. Reuse rstml recovery and the existing UI lowering.
use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote, quote_spanned};
use rstml::{
    Parser, ParserConfig,
    node::{KeyedAttributeValue, Node, NodeAttribute},
};
use syn::spanned::Spanned;

pub fn lower(input: TokenStream) -> syn::Result<TokenStream> {
    let (nodes, diagnostics) = Parser::new(
        ParserConfig::default()
            .recover_block(true)
            .number_of_top_level_nodes(1),
    )
    .parse_recoverable(input)
    .split_vec();
    let mut output = TokenStream::new();
    for diagnostic in diagnostics {
        output.extend(diagnostic.emit_as_item_tokens());
    }
    for node in nodes {
        lower_node(node, &mut output)?;
    }
    Ok(output)
}

fn lower_node(node: Node, output: &mut TokenStream) -> syn::Result<()> {
    match node {
        Node::Element(element) => {
            let name = element.name();
            let mut fields = TokenStream::new();
            let mut methods = TokenStream::new();
            for attribute in element.attributes() {
                let NodeAttribute::Attribute(attribute) = attribute else {
                    return Err(syn::Error::new_spanned(
                        attribute,
                        "RSX experiment requires named props",
                    ));
                };
                let key = &attribute.key;
                let value = match &attribute.possible_value {
                    KeyedAttributeValue::Value(value) => {
                        prop_expression(value.value.to_token_stream())
                    }
                    KeyedAttributeValue::None => quote!(true),
                    KeyedAttributeValue::Binding(_) => {
                        return Err(syn::Error::new_spanned(
                            attribute,
                            "use a Rust closure in braces",
                        ));
                    }
                };
                if key.to_string() == "style" {
                    methods.extend(quote!(.#key(#value)));
                } else {
                    fields.extend(
                        quote_spanned!(value.span()=> #key: ::core::convert::Into::into(#value),),
                    );
                }
            }
            let local = Ident::new("__actor_rsx_parent", Span::mixed_site());
            let parent = quote!({
                #[allow(clippy::needless_update)]
                let #local = #name { #fields ..::core::default::Default::default() };
                #local #methods
            });
            let mut children = TokenStream::new();
            for child in element.children.clone() {
                lower_node(child, &mut children)?;
            }
            if children.is_empty() {
                output.extend(quote!(ui!(#parent);));
            } else {
                output.extend(quote!(ui!(#parent, { #children });));
            }
            if let Some(close) = &element.close_tag {
                // A type-only source anchor makes closing names visible to LSP.
                let name = &close.name;
                output.extend(
                    quote!(let _: ::core::option::Option<#name> = ::core::option::Option::None;),
                );
            }
        }
        Node::Block(block) => output.extend(quote!(ui!(#block);)),
        Node::Text(text) => {
            let value = text.value;
            output.extend(quote!(ui!(::blockui::Text::new(#value));));
        }
        Node::Comment(_) => {}
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "RSX experiment supports component tags, quoted text and Rust blocks",
            ));
        }
    }
    Ok(())
}

// RSX braces delimit prop expressions. Remove that delimiter for a single
// expression so generated Rust does not warn about braces around arguments.
// Keep real statement blocks and invalid inputs intact, including their spans.
fn prop_expression(tokens: TokenStream) -> TokenStream {
    let parts: Vec<_> = tokens.clone().into_iter().collect();
    if let [TokenTree::Group(group)] = parts.as_slice()
        && group.delimiter() == Delimiter::Brace
        && syn::parse2::<syn::Expr>(group.stream()).is_ok()
    {
        return group.stream();
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovery_preserves_invalid_rust_and_reports_unclosed_tags() {
        let output = lower(quote!(<Column><Text text={self.count.} />))
            .unwrap()
            .to_string();
        assert!(output.contains("self . count ."), "{output}");
        assert!(output.contains("compile_error"), "{output}");
        assert!(output.contains("Column"), "{output}");
    }
    #[test]
    fn props_and_types_keep_tokens_in_typed_rust_positions() {
        let output = lower(quote!(<Column gap={8.0}><Text text="Hello" /></Column>))
            .unwrap()
            .to_string();
        assert!(output.contains("Column { gap :"), "{output}");
        assert!(output.contains("Text { text :"), "{output}");
        assert!(!output.contains("compile_error"), "{output}");
    }
}
