//! Token-preserving block sugar: incomplete expressions remain visible to LSP.
use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::quote;

/// Detect the explicit marker without parsing expressions or interpreting other
/// macros' token languages. Nested items opt in independently.
pub(super) fn has_ui_marker(input: TokenStream) -> bool {
    use syn::parse::Parser;
    let tokens: Vec<_> = input.into_iter().collect();
    let mut index = 0;
    while index < tokens.len() {
        if matches!(&tokens[index], TokenTree::Ident(name)
            if matches!(name.to_string().as_str(), "fn" | "struct" | "enum" | "impl" | "mod" | "trait" | "union" | "type" | "static" | "use" | "macro_rules"))
        {
            let parser = |input: syn::parse::ParseStream<'_>| {
                if input.peek(syn::Token![fn]) {
                    crate::opaque_impl::function(input)?;
                } else {
                    input.parse::<syn::Item>()?;
                }
                input.parse::<TokenStream>()
            };
            if let Ok(rest) = parser.parse2(tokens[index..].iter().cloned().collect()) {
                index = tokens.len() - rest.into_iter().count();
                continue;
            }
        }
        if matches!(&tokens[index], TokenTree::Ident(name) if name == "ui" || (cfg!(feature = "rsx") && name == "rsx"))
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!')
        {
            return true;
        }
        if matches!(&tokens[index], TokenTree::Punct(p) if p.as_char() == '#')
            && let Some(TokenTree::Group(attribute)) = tokens.get(index + 1)
            && attribute.delimiter() == Delimiter::Bracket
            && matches!(attribute.stream().into_iter().next(), Some(TokenTree::Ident(name)) if name == "ui")
        {
            return true;
        }
        if let TokenTree::Group(group) = &tokens[index]
            && !(index > 0
                && matches!(&tokens[index - 1], TokenTree::Punct(p) if p.as_char() == '!'))
            && has_ui_marker(group.stream())
        {
            return true;
        }
        index += 1;
    }
    false
}

pub fn expand(builder: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let tree = builder.to_string() == "tree";
    let block = builder.to_string() == "block";
    if !builder.is_empty() && !tree && !block {
        return Err(syn::Error::new_spanned(
            builder,
            "#[view] takes no arguments, block for UI tags, or tree for a single root expression",
        ));
    }
    // Parse only the signature. Incomplete body expressions must reach LSP.
    let mut tokens: Vec<_> = input.into_iter().collect();
    let Some(TokenTree::Group(body)) = tokens.pop() else {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[view] requires a function body",
        ));
    };
    if body.delimiter() != Delimiter::Brace {
        return Err(syn::Error::new(
            body.span(),
            "#[view] requires a function body",
        ));
    }
    let header: TokenStream = tokens.into_iter().collect();
    let mut function: syn::ImplItemFn = syn::parse2(quote!(#header {}))?;
    if function.sig.asyncness.is_some()
        || function.sig.constness.is_some()
        || function.sig.unsafety.is_some()
    {
        return Err(syn::Error::new_spanned(
            &function.sig,
            "#[view] requires a synchronous, safe, non-const function",
        ));
    }
    let missing_return = matches!(function.sig.output, syn::ReturnType::Default);
    let diagnostic = if missing_return {
        syn::Error::new_spanned(
            &function.sig,
            "#[view] requires an explicit View or Result<View, E> return type, where E: From<blockui::Error>",
        ).to_compile_error()
    } else {
        TokenStream::new()
    };
    if tree || block {
        if function.sig.inputs.iter().any(|argument| matches!(argument,
            syn::FnArg::Typed(argument) if matches!(argument.ty.as_ref(),
                syn::Type::Reference(reference) if matches!(reference.elem.as_ref(),
                    syn::Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Ui")))))
        {
            return Err(syn::Error::new_spanned(&function.sig, "#[view(block)] and #[view(tree)] own their builder and take no Ui parameter"));
        }
        let builder = Ident::new("__actor_tree_builder", Span::mixed_site());
        let mut next = 0;
        let contents = if tree {
            rewrite_tree(body.stream(), &builder, &mut next)?
        } else {
            body.stream()
        };
        let contents = rewrite(contents, &builder, &mut next)?;
        let build = if tree {
            quote!(#builder.add({ #contents });)
        } else {
            quote!({ #contents };)
        };
        let attrs = &function.attrs;
        let visibility = &function.vis;
        let signature = &function.sig;
        return Ok(quote! {
            #diagnostic
            #(#attrs)* #visibility #signature {
                let #builder = ::blockui::Ui::new();
                #build
                ::blockui::__finish_view(#builder)
            }
        });
    }
    let argument = declared_builder(&function.sig)?.clone();
    let syn::Pat::Ident(pattern) = argument.pat.as_ref() else {
        unreachable!()
    };
    let builder = &pattern.ident;
    function.sig.inputs.pop();
    let contents = rewrite(body.stream(), builder, &mut 0)?;
    let storage = Ident::new("__actor_ui_storage", Span::mixed_site());
    // Keep incomplete bodies visible to rust-analyzer even when the signature is invalid.
    let finish = if missing_return {
        TokenStream::new()
    } else {
        quote::quote_spanned!(syn::spanned::Spanned::span(&function.sig.output)=>
            ::blockui::__finish_view(#storage)
        )
    };
    let attrs = &function.attrs;
    let visibility = &function.vis;
    let signature = &function.sig;
    Ok(quote! {
        #diagnostic
        #(#attrs)* #visibility #signature {
            let #storage = ::blockui::Ui::new();
            let #argument = &#storage;
            { #contents };
            #finish
        }
    })
}

/// A builder parameter is explicit in source; only the generated wrapper owns it.
pub(super) fn declared_builder(signature: &syn::Signature) -> syn::Result<&syn::PatType> {
    let Some(syn::FnArg::Typed(argument)) = signature.inputs.last() else {
        return Err(syn::Error::new_spanned(
            signature,
            "#[view] requires a final builder parameter such as ui: &Ui",
        ));
    };
    let valid_type = matches!(argument.ty.as_ref(), syn::Type::Reference(reference)
        if reference.mutability.is_none() && matches!(reference.elem.as_ref(), syn::Type::Path(path)
            if path.qself.is_none() && path.path.segments.last().is_some_and(|segment| segment.ident == "Ui")));
    let valid_name = matches!(argument.pat.as_ref(), syn::Pat::Ident(pattern)
        if pattern.by_ref.is_none() && pattern.mutability.is_none() && pattern.subpat.is_none());
    if !valid_type || !valid_name {
        return Err(syn::Error::new_spanned(
            argument,
            "#[view] requires a final builder parameter such as ui: &Ui",
        ));
    }
    Ok(argument)
}

fn rewrite(input: TokenStream, builder: &Ident, next: &mut usize) -> syn::Result<TokenStream> {
    let tokens: Vec<_> = input.into_iter().collect();
    let mut output = TokenStream::new();
    let mut index = 0;
    while index < tokens.len() {
        if matches!(&tokens[index], TokenTree::Ident(name) if name == "live")
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!')
            && let Some(TokenTree::Group(expression)) = tokens.get(index + 2)
            && expression.delimiter() == Delimiter::Parenthesis
            && !matches!(index.checked_sub(1).and_then(|i| tokens.get(i)), Some(TokenTree::Punct(p)) if p.as_char() == ':')
        {
            let receiver = Ident::new("self", tokens[index].span());
            let argument = Ident::new("__actor_live_receiver", Span::mixed_site());
            let expression = live_receiver(expression.stream(), &argument);
            output.extend(quote!(#receiver.live(|#argument| { #expression })));
            index += 3;
            continue;
        }
        #[cfg(feature = "rsx")]
        if matches!(&tokens[index], TokenTree::Ident(name) if name == "rsx")
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!')
            && let Some(TokenTree::Group(arguments)) = tokens.get(index + 2)
            && !matches!(index.checked_sub(1).and_then(|i| tokens.get(i)), Some(TokenTree::Punct(p)) if p.as_char() == ':')
        {
            let lowered = crate::rsx::lower(arguments.stream())?;
            output.extend(rewrite(lowered, builder, next)?);
            index += 3;
            if matches!(tokens.get(index), Some(TokenTree::Punct(p)) if p.as_char() == ';') {
                index += 1;
            }
            continue;
        }
        // Consume UI marker inputs ourselves, preserving incomplete expressions
        // and source spans rather than parsing them as syn::Expr.
        if matches!(&tokens[index], TokenTree::Ident(name) if name == "ui")
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!')
            && let Some(TokenTree::Group(arguments)) = tokens.get(index + 2)
            && matches!(
                arguments.delimiter(),
                Delimiter::Parenthesis | Delimiter::Brace
            )
            && !matches!(index.checked_sub(1).and_then(|i| tokens.get(i)), Some(TokenTree::Punct(p)) if p.as_char() == ':')
        {
            let mut parts: Vec<_> = arguments.stream().into_iter().collect();
            if matches!(parts.last(), Some(TokenTree::Punct(p)) if p.as_char() == ',') {
                parts.pop();
            }
            if parts.is_empty() {
                return Err(syn::Error::new(
                    arguments.span(),
                    "ui! requires a widget expression",
                ));
            }
            // Only a final `, { children }` denotes a container. Commas in
            // constructor calls and generic arguments remain expression tokens.
            let children = match parts.as_slice() {
                [
                    parent @ ..,
                    TokenTree::Punct(comma),
                    TokenTree::Group(children),
                ] if comma.as_char() == ',' && children.delimiter() == Delimiter::Brace => {
                    Some((parent.iter().cloned().collect::<TokenStream>(), children))
                }
                _ => None,
            };
            if let Some((parent, children)) = children {
                let parent = rewrite(parent, builder, next)?;
                let parent = ui_identity(parent, next);
                output.extend(container(
                    parent,
                    children.stream(),
                    children.span(),
                    builder,
                    next,
                )?);
            } else {
                let expression = compact_children(parts.into_iter().collect(), next)?;
                let expression = rewrite(expression, builder, next)?;
                let expression = ui_identity(expression, next);
                output.extend(quote!(#builder.add(#expression);));
            }
            index += 3;
            if matches!(tokens.get(index), Some(TokenTree::Punct(p)) if p.as_char() == ';') {
                index += 1;
            }
            continue;
        }
        // Nested items own their attributes and bodies. In particular an inner
        // #[view] must build with its own parameter, never the outer builder.
        if matches!(&tokens[index], TokenTree::Ident(name)
            if matches!(name.to_string().as_str(), "fn" | "struct" | "enum" | "impl" | "mod" | "trait" | "union" | "type" | "static" | "use" | "macro_rules"))
        {
            use syn::parse::Parser;
            let remaining: TokenStream = tokens[index..].iter().cloned().collect();
            let parser = |input: syn::parse::ParseStream<'_>| {
                if input.peek(syn::Token![fn]) {
                    // Preserve unfinished nested function bodies for their own
                    // macros and LSP; the outer builder must not rewrite them.
                    crate::opaque_impl::function(input)?;
                } else {
                    input.parse::<syn::Item>()?;
                }
                input.parse::<TokenStream>()
            };
            if let Ok(rest) = parser.parse2(remaining) {
                let end = tokens.len() - rest.into_iter().count();
                output.extend(tokens[index..end].iter().cloned());
                index = end;
                continue;
            }
        }
        if matches!(&tokens[index], TokenTree::Punct(p) if p.as_char() == '#')
            && let Some(TokenTree::Group(attribute)) = tokens.get(index + 1)
        {
            let parts: Vec<_> = attribute.stream().into_iter().collect();
            if attribute.delimiter() == Delimiter::Bracket
                && matches!(parts.first(), Some(TokenTree::Ident(name)) if name == "node" || name == "ui")
            {
                let ui_marker =
                    matches!(parts.first(), Some(TokenTree::Ident(name)) if name == "ui");
                if parts.len() == 2
                    && let TokenTree::Group(parent) = &parts[1]
                    && parent.delimiter() == Delimiter::Parenthesis
                {
                    let Some(TokenTree::Group(block)) = tokens.get(index + 2)
                        .filter(|token| matches!(token, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace))
                    else {
                        let message = if ui_marker { "#[ui(parent)] requires a child block" } else { "#[node(parent)] requires a child block" };
                        return Err(syn::Error::new(attribute.span(), message));
                    };
                    if parent.stream().is_empty() {
                        return Err(syn::Error::new(
                            parent.span(),
                            "expected a parent expression",
                        ));
                    }
                    let parent = if ui_marker {
                        ui_identity(parent.stream(), next)
                    } else {
                        parent.stream()
                    };
                    output.extend(container(
                        parent,
                        block.stream(),
                        block.span(),
                        builder,
                        next,
                    )?);
                    index += 3;
                    continue;
                }
                if parts.len() != 1 {
                    let message = if ui_marker {
                        "use #[ui] for a widget statement, or #[ui(parent)] with a child block"
                    } else {
                        "use #[node] for a widget statement, or #[node(parent)] with a child block"
                    };
                    return Err(syn::Error::new(attribute.span(), message));
                }
                if let Some(TokenTree::Group(block)) = tokens.get(index + 2)
                    && block.delimiter() == Delimiter::Brace
                {
                    let content: Vec<_> = block.stream().into_iter().collect();
                    let split = content
                        .iter()
                        .position(
                            |token| matches!(token, TokenTree::Punct(p) if p.as_char() == ';'),
                        )
                        .ok_or_else(|| {
                            syn::Error::new(
                                block.span(),
                                if ui_marker {
                                    "a #[ui] block starts with a parent expression followed by ;"
                                } else {
                                    "a #[node] block starts with a parent expression followed by ;"
                                },
                            )
                        })?;
                    if split == 0
                        || matches!(content.first(), Some(TokenTree::Ident(name)) if name == "let")
                    {
                        return Err(syn::Error::new(
                            block.span(),
                            if ui_marker {
                                "a #[ui] block starts with its parent widget expression"
                            } else {
                                "a #[node] block starts with its parent widget expression"
                            },
                        ));
                    }
                    let parent: TokenStream = content[..split].iter().cloned().collect();
                    let children: TokenStream = content[split + 1..].iter().cloned().collect();
                    output.extend(container(parent, children, block.span(), builder, next)?);
                    index += 3;
                    continue;
                }
                let start = index + 2;
                let end = (start..tokens.len()).find(|&position|
                            matches!(&tokens[position], TokenTree::Punct(p) if p.as_char() == ';'))
                            .ok_or_else(|| syn::Error::new(attribute.span(), "#[node] requires an expression followed by ;"))?;
                if start == end
                    || matches!(tokens.get(start), Some(TokenTree::Ident(name))
                            if matches!(name.to_string().as_str(), "let" | "return" | "break" | "continue" | "fn" | "struct" | "impl"))
                {
                    return Err(syn::Error::new(
                        attribute.span(),
                        "#[node] marks a widget expression, not a declaration or control statement",
                    ));
                }
                // Do not parse the expression: `Text::` must still expand
                // while a developer is requesting completion.
                let expression =
                    rewrite(tokens[start..end].iter().cloned().collect(), builder, next)?;
                let expression = if ui_marker {
                    ui_identity(expression, next)
                } else {
                    with_local_identity(expression)
                };
                output.extend(quote!(#builder.add(#expression);));
                index = end + 1;
                continue;
            }
        }
        let token = match &tokens[index] {
            // Macro inputs are opaque token languages. Their owner, not this
            // attribute, decides whether an inner #[node] is meaningful.
            TokenTree::Group(group)
                if index > 0
                    && matches!(&tokens[index - 1], TokenTree::Punct(p) if p.as_char() == '!') =>
            {
                TokenTree::Group(group.clone())
            }
            TokenTree::Group(group) => {
                let mut replacement =
                    Group::new(group.delimiter(), rewrite(group.stream(), builder, next)?);
                replacement.set_span(group.span());
                TokenTree::Group(replacement)
            }
            token => token.clone(),
        };
        output.extend([token]);
        index += 1;
    }
    Ok(output)
}

fn ui_identity(expression: TokenStream, next: &mut usize) -> TokenStream {
    let name = syn::LitStr::new(&format!("ui.node.{}", *next), Span::mixed_site());
    *next += 1;
    quote!(::blockui::IntoView::into_view(#expression).__local_identity(#name))
}

// Tree interpretation is opt-in. Ordinary #[view] must leave user structs
// containing a real children field untouched.
fn rewrite_tree(input: TokenStream, builder: &Ident, next: &mut usize) -> syn::Result<TokenStream> {
    let tokens: Vec<_> = input.into_iter().collect();
    let mut output = TokenStream::new();
    for (index, token) in tokens.iter().enumerate() {
        if let TokenTree::Group(group) = token {
            if index > 0 && matches!(&tokens[index - 1], TokenTree::Punct(p) if p.as_char() == '!')
            {
                output.extend([token.clone()]);
                continue;
            }
            if group.delimiter() == Delimiter::Brace
                && index > 0
                && matches!(&tokens[index - 1], TokenTree::Ident(_))
                && let Some((fields, children)) = child_fields(group.stream())?
            {
                let mut fields = Group::new(Delimiter::Brace, fields);
                fields.set_span(group.span());
                let children = child_expressions(children)
                    .into_iter()
                    .map(|child| {
                        rewrite_tree(child, builder, next)
                            .and_then(|child| rewrite(child, builder, next))
                            .map(|child| quote!(::blockui::IntoView::into_view(#child)))
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                output.extend(quote!(#fields.children(::std::vec![#(#children),*])));
                continue;
            }
            let mut replacement = Group::new(
                group.delimiter(),
                rewrite_tree(group.stream(), builder, next)?,
            );
            replacement.set_span(group.span());
            output.extend([TokenTree::Group(replacement)]);
        } else {
            output.extend([token.clone()]);
        }
    }
    Ok(output)
}

// Strip only the explicit `children: [...]` field. Keep all real widget fields
// and unfinished child expressions visible to rust-analyzer.
fn child_fields(input: TokenStream) -> syn::Result<Option<(TokenStream, TokenStream)>> {
    let tokens: Vec<_> = input.into_iter().collect();
    let mut found = None;
    for index in 0..tokens.len() {
        if (index == 0 || matches!(&tokens[index - 1], TokenTree::Punct(p) if p.as_char() == ','))
            && matches!(&tokens[index], TokenTree::Ident(name) if name == "children")
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':')
            && let Some(TokenTree::Group(children)) = tokens.get(index + 2)
            && children.delimiter() == Delimiter::Bracket
        {
            if found.is_some() {
                return Err(syn::Error::new(
                    tokens[index].span(),
                    "duplicate children field",
                ));
            }
            found = Some((index, children.stream()));
        }
    }
    Ok(found.map(|(index, children)| {
        let end = index
            + 3
            + usize::from(
                matches!(tokens.get(index + 3), Some(TokenTree::Punct(p)) if p.as_char() == ','),
            );
        let fields = tokens[..index]
            .iter()
            .chain(tokens[end..].iter())
            .cloned()
            .collect();
        (fields, children)
    }))
}

fn child_expressions(input: TokenStream) -> Vec<TokenStream> {
    use syn::parse::Parser;
    if let Ok(expressions) =
        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
            .parse2(input.clone())
    {
        return expressions
            .into_iter()
            .map(|expression| quote!(#expression))
            .collect();
    }
    // Recovery for completion at `Text::` or `self.`. Groups are opaque here,
    // so commas in function calls and nested children do not split a child.
    let mut children = Vec::new();
    let mut child = TokenStream::new();
    for token in input {
        if matches!(&token, TokenTree::Punct(p) if p.as_char() == ',') {
            children.push(std::mem::take(&mut child));
        } else {
            child.extend([token]);
        }
    }
    if !child.is_empty() {
        children.push(child);
    }
    children
}

// Preserve incomplete expressions and source spans for rust-analyzer. The
// function-pointer parameter rejects implicit captures of view-local values.
fn live_receiver(input: TokenStream, receiver: &Ident) -> TokenStream {
    input
        .into_iter()
        .map(|token| match token {
            TokenTree::Ident(name) if name == "self" => {
                let mut replacement = receiver.clone();
                replacement.set_span(name.span().resolved_at(receiver.span()));
                TokenTree::Ident(replacement)
            }
            TokenTree::Group(group) => {
                let mut replacement =
                    Group::new(group.delimiter(), live_receiver(group.stream(), receiver));
                replacement.set_span(group.span());
                TokenTree::Group(replacement)
            }
            token => token,
        })
        .collect()
}

// Match #[node]'s existing hint for named leaves/parents. Constructors keep their
// semantic identities; dynamic collections still require explicit keys.
fn with_local_identity(expression: TokenStream) -> TokenStream {
    let mut tokens: Vec<_> = expression.clone().into_iter().collect();
    // A clone keeps the authored instance name; the allocation is ephemeral.
    let cloned = tokens.len() >= 4
        && matches!(&tokens[tokens.len()-3], TokenTree::Punct(p) if p.as_char() == '.')
        && matches!(&tokens[tokens.len()-2], TokenTree::Ident(name) if name == "clone")
        && matches!(&tokens[tokens.len()-1], TokenTree::Group(args)
            if args.delimiter() == Delimiter::Parenthesis && args.stream().is_empty());
    if cloned {
        tokens.truncate(tokens.len() - 3);
    }
    let named = !tokens.is_empty()
        && tokens.iter().enumerate().all(|(index, token)| {
            if index % 2 == 0 {
                matches!(token, TokenTree::Ident(_))
            } else {
                matches!(token, TokenTree::Punct(p) if p.as_char() == '.')
            }
        })
        && tokens.len() % 2 == 1;
    if named {
        let name: TokenStream = tokens.into_iter().collect();
        quote!(::blockui::IntoView::into_view(#expression).__local_identity(::core::stringify!(#name)))
    } else {
        expression
    }
}

fn container(
    parent: TokenStream,
    body: TokenStream,
    span: Span,
    builder: &Ident,
    next: &mut usize,
) -> syn::Result<TokenStream> {
    let guard = Ident::new(&format!("__actor_ui_guard_{}", *next), Span::mixed_site());
    *next += 1;
    let children = rewrite(body, builder, next)?;
    let parent = with_local_identity(parent);
    let mut block = Group::new(
        Delimiter::Brace,
        quote! {
            let #guard = #builder.scope(#parent);
            #children
        },
    );
    block.set_span(span);
    Ok(TokenStream::from(TokenTree::Group(block)))
}

// `.children({ ... })` explicitly marks child lists, so ordinary expression
// blocks retain normal Rust semantics. Keep other macro token languages opaque.
fn compact_children(input: TokenStream, next: &mut usize) -> syn::Result<TokenStream> {
    let tokens: Vec<_> = input.into_iter().collect();
    let mut output = TokenStream::new();
    let mut index = 0;
    while index < tokens.len() {
        if matches!(&tokens[index], TokenTree::Punct(p) if p.as_char() == '.')
            && matches!(tokens.get(index + 1), Some(TokenTree::Ident(name)) if name == "children")
            && let Some(TokenTree::Group(args)) = tokens.get(index + 2)
            && args.delimiter() == Delimiter::Parenthesis
            && let [TokenTree::Group(block)] =
                args.stream().into_iter().collect::<Vec<_>>().as_slice()
            && block.delimiter() == Delimiter::Brace
        {
            let rows = block.stream();
            let mut statement = TokenStream::new();
            let mut body = TokenStream::new();
            let list = Ident::new("__actor_compact_children", Span::mixed_site());
            let mut emit = |statement: TokenStream, body: &mut TokenStream| -> syn::Result<()> {
                if statement.is_empty() {
                    return Ok(());
                }
                if matches!(statement.clone().into_iter().next(), Some(TokenTree::Ident(name)) if name == "let")
                {
                    let statement = compact_children(statement, next)?;
                    body.extend(quote!(#statement;));
                } else {
                    let expression = compact_children(statement, next)?;
                    let expression = ui_identity(expression, next);
                    body.extend(quote!(#list.push(::blockui::IntoView::into_view(#expression));));
                }
                Ok(())
            };
            for token in rows {
                if matches!(&token, TokenTree::Punct(p) if p.as_char() == ';') {
                    emit(std::mem::take(&mut statement), &mut body)?;
                } else {
                    statement.extend([token]);
                }
            }
            emit(statement, &mut body)?;
            output.extend(quote!(.children({
                let mut #list = ::std::vec::Vec::<::blockui::View>::new();
                #body
                #list
            })));
            index += 3;
            continue;
        }
        match &tokens[index] {
            TokenTree::Group(group)
                if !(index > 0
                    && matches!(&tokens[index-1], TokenTree::Punct(p) if p.as_char() == '!')) =>
            {
                let mut group_out =
                    Group::new(group.delimiter(), compact_children(group.stream(), next)?);
                group_out.set_span(group.span());
                output.extend([TokenTree::Group(group_out)]);
            }
            token => output.extend([token.clone()]),
        }
        index += 1;
    }
    Ok(output)
}

/// Standalone roots use the same builder and marker lowering as tagged views.
pub fn expand_ui(input: TokenStream) -> syn::Result<TokenStream> {
    let builder = Ident::new("__actor_ui_macro_builder", Span::mixed_site());
    let contents = rewrite(quote!(ui!(#input)), &builder, &mut 0)?;
    Ok(quote!({
        let #builder = ::blockui::Ui::new();
        #contents
        ::blockui::__finish_view(#builder)
    }))
}

#[cfg(feature = "rsx")]
pub fn expand_rsx(input: TokenStream) -> syn::Result<TokenStream> {
    let builder = Ident::new("__actor_rsx_builder", Span::mixed_site());
    let contents = rewrite(crate::rsx::lower(input)?, &builder, &mut 0)?;
    Ok(quote!({
        let #builder = ::blockui::Ui::new();
        #contents
        ::blockui::__finish_view(#builder)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_tags_preserve_incomplete_expressions_and_ordinary_statements() {
        let expanded = expand(
            TokenStream::new(),
            quote! {
                fn view(tree: &Ui) -> View {
                    prepare();
                    #[node] {
                        Column { gap: 12.0, ..default() };
                        let label = Text::new("hello");
                        #[node] label;
                        if visible { #[node] Text::; }
                        #[node] {
                            Row::default();
                            #[node] Button { enabled: true, ..default() }.style(style);
                        }
                    }
                }
            },
        )
        .unwrap()
        .to_string();
        assert!(expanded.contains("let tree : & Ui = & __actor_ui_storage"));
        assert!(expanded.contains("fn view ()"));
        assert!(expanded.contains("prepare () ;"));
        assert!(!expanded.contains("add (prepare"));
        assert!(expanded.contains("tree . scope (Column { gap : 12.0"));
        assert!(expanded.contains("tree . scope (Row :: default ())"));
        assert!(expanded.contains("into_view (label)"));
        assert!(expanded.contains("tree . add (Text ::)"));
        assert!(expanded.contains("Button { enabled : true"));
        assert!(expanded.ends_with(":: blockui :: __finish_view (__actor_ui_storage) }"));
    }

    #[test]
    fn unfinished_nested_view_keeps_its_own_node_attributes() {
        let nested = quote! {
            #[blockui::view]
            fn inner(inner_ui: &Ui) -> View {
                #[node] Text::;
            }
        };
        let expanded = expand(
            TokenStream::new(),
            quote! {
                fn outer(outer_ui: &Ui) -> View {
                    #nested
                    #[node] Text::new("outer");
                }
            },
        )
        .unwrap()
        .to_string();
        assert!(expanded.contains(&nested.to_string()), "{expanded}");
        assert!(expanded.contains("outer_ui . add (Text :: new"));
    }

    #[test]
    fn unfinished_functions_inside_nested_items_keep_their_builders() {
        for nested in [
            quote! {
                impl Helper {
                    #[blockui::view]
                    fn inner(&self, inner_ui: &Ui) -> View {
                        #[node] Text::;
                    }
                }
            },
            quote! {
                mod helper {
                    #[blockui::view]
                    pub fn inner(inner_ui: &Ui) -> View {
                        #[node] Text::;
                    }
                }
            },
            quote! {
                trait Helper {
                    #[blockui::view]
                    fn inner(&self, inner_ui: &Ui) -> View {
                        #[node] Text::;
                    }
                }
            },
        ] {
            let expanded = expand(
                TokenStream::new(),
                quote! {
                    fn outer(outer_ui: &Ui) -> View {
                        #nested
                        #[node] Text::new("outer");
                    }
                },
            )
            .unwrap()
            .to_string();
            assert!(expanded.contains(&nested.to_string()), "{expanded}");
            assert!(expanded.contains("outer_ui . add (Text :: new"));
        }
    }

    #[test]
    fn macro_inputs_remain_opaque() {
        let expanded = expand(
            TokenStream::new(),
            quote! {
                fn example(ui: &Ui) -> View {
                    custom! { #[node] Text::new("owned by custom"); }
                    #[node] Text::new("owned by view");
                }
            },
        )
        .unwrap()
        .to_string();
        assert!(expanded.contains("custom ! { # [node] Text :: new"));
        assert!(expanded.contains("ui . add (Text :: new"));
    }

    #[test]
    fn missing_return_type_explains_the_builder_contract() {
        let expanded = expand(
            quote!(),
            quote!(
                fn panel(ui: &Ui) {
                    #[node]
                    Text::new("Hello");
                }
            ),
        )
        .unwrap()
        .to_string();
        assert!(expanded.contains("requires an explicit View or Result<View, E>"));
        assert!(expanded.contains("ui . add (Text :: new"));
    }

    #[test]
    fn rejects_implicit_builder_and_extra_syntax() {
        for (attributes, function) in [
            (
                quote!(),
                quote!(
                    fn view() -> View {}
                ),
            ),
            (
                quote!(tree),
                quote!(
                    fn view(tree: &Ui) -> View {}
                ),
            ),
            (
                quote!(),
                quote!(
                    fn view(tree: &Ui) -> View {
                        #[node(Column::default())]
                        Text::new("missing child block");
                    }
                ),
            ),
            (
                quote!(),
                quote!(
                    fn view(tree: &Ui) -> View {
                        #[node]
                        let text = Text::new("hello");
                    }
                ),
            ),
            (
                quote!(),
                quote!(
                    fn view(tree: &Ui) -> View {
                        #[node]
                        Text::new("hello")
                    }
                ),
            ),
            (
                quote!(),
                quote!(
                    fn view(tree: &Ui) -> View {
                        #[node]
                        {}
                    }
                ),
            ),
        ] {
            assert!(expand(attributes, function).is_err());
        }
    }
}

#[cfg(test)]
mod tag_diagnostics {
    use super::*;
    #[test]
    fn ui_diagnostics_name_the_tag_used_in_source() {
        let error = expand(
            quote!(block),
            quote!(
                fn page() -> View {
                    #[ui(Column::default())]
                    Text::new("missing child block");
                }
            ),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("#[ui(parent)] requires a child block")
        );
        let error = expand(
            quote!(block),
            quote!(
                fn page() -> View {
                    #[ui = "bad"]
                    Text::new("invalid tag");
                }
            ),
        )
        .unwrap_err();
        assert!(error.to_string().contains("use #[ui]"));
    }
}

#[cfg(test)]
mod macro_ui_tests {
    use super::*;

    #[test]
    fn markers_preserve_unfinished_tokens_and_leave_other_macros_opaque() {
        let output = expand_ui(quote!(Column::, {
            ui!(Text::);
            ui!(other!(ui!(untouched)));
        },))
        .unwrap()
        .to_string();
        assert!(output.contains("Column ::"));
        assert!(output.contains("Text ::"));
        assert!(output.contains("other ! (ui ! (untouched))"));
        assert!(expand_ui(TokenStream::new()).is_err());
    }
}
