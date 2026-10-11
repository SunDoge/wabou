//! Resolve emitted SDK paths without changing token spans or parsing view bodies.
use proc_macro2::{Group, Ident, TokenStream, TokenTree};

pub(super) fn resolve(output: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let name = ["blockui", "wabou-native"].into_iter().find_map(|package| {
        match proc_macro_crate::crate_name(package).ok()? {
            proc_macro_crate::FoundCrate::Name(name) => Some(name),
            // The SDK already declares `extern crate self as blockui`.
            proc_macro_crate::FoundCrate::Itself => Some("blockui".into()),
        }
    });
    match name {
        Some(name) if name != "blockui" => rewrite(output.into(), &name).into(),
        _ => output,
    }
}

fn rewrite(tokens: TokenStream, sdk: &str) -> TokenStream {
    let tokens: Vec<_> = tokens.into_iter().collect();
    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            match token {
                TokenTree::Group(group) => {
                    let mut result = Group::new(group.delimiter(), rewrite(group.stream(), sdk));
                    result.set_span(group.span());
                    TokenTree::Group(result)
                }
                TokenTree::Ident(name)
                    if name == "blockui"
                // Authored identifiers retain their source spelling. In particular,
                // an associated type named blockui is not an SDK reference.
                && name.span().source_text().as_deref() != Some("blockui")
                && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':')
                && matches!(tokens.get(index + 2), Some(TokenTree::Punct(p)) if p.as_char() == ':')
                && sdk_root(&tokens, index) =>
                {
                    TokenTree::Ident(Ident::new(sdk, name.span()))
                }
                token => token.clone(),
            }
        })
        .collect()
}

// Do not rename a segment of an application path such as helpers::blockui.
// Count colons because `field: ::blockui` has three, whereas a path has two.
// A lifetime or keyword may immediately precede an absolute type path.
fn sdk_root(tokens: &[TokenTree], index: usize) -> bool {
    let mut start = index;
    while start > 0 && matches!(&tokens[start - 1], TokenTree::Punct(p) if p.as_char() == ':') {
        start -= 1;
    }
    if index - start != 2 || start == 0 {
        return true;
    }
    match &tokens[start - 1] {
        TokenTree::Ident(name) => {
            matches!(
                name.to_string().as_str(),
                "impl"
                    | "as"
                    | "dyn"
                    | "return"
                    | "for"
                    | "in"
                    | "mut"
                    | "const"
                    | "match"
                    | "if"
                    | "while"
                    | "break"
                    | "yield"
            ) || (start >= 2
                && matches!(&tokens[start - 2], TokenTree::Punct(p) if p.as_char() == '\''))
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn resolves_paths_and_nested_attributes_without_rewriting_other_identifiers() {
        let actual = rewrite(
            quote! {
                #[blockui::view]
                fn blockui() { let blockui = "blockui"; ::blockui::Ui::new(); module::blockui::caption(); }
                type Bound<'host> = &'host ::blockui::Ui;
            },
            "native",
        )
        .to_string();
        let expected = quote! {
            #[native::view]
            fn blockui() { let blockui = "blockui"; ::native::Ui::new(); module::blockui::caption(); }
            type Bound<'host> = &'host ::native::Ui;
        }
        .to_string();
        assert_eq!(actual, expected);
    }
}
