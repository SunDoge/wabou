//! Function authoring over the same statically declared, runtime-owned schema.
use quote::quote;
use syn::{ItemFn, Type, spanned::Spanned};

pub(super) fn expand(
    mut function: ItemFn,
    attributes: proc_macro2::TokenStream,
    root: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    if function.sig.asyncness.is_some()
        || function.sig.unsafety.is_some()
        || !function.sig.generics.params.is_empty()
    {
        return Err(syn::Error::new(
            function.sig.span(),
            "component functions must be synchronous and nongeneric",
        ));
    }
    let name = &function.sig.ident;
    let visibility = &function.vis;
    let output = &function.sig.output;
    let mut fields = Vec::new();
    let mut bound_arguments = Vec::new();
    let mut descriptor_arguments = Vec::new();
    let mut contexts = Vec::new();
    let block_view = function.attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "view")
    });
    if block_view {
        super::view_blocks::declared_builder(&function.sig)?;
    }
    let parameter_count = function.sig.inputs.len();
    for (index, argument) in function.sig.inputs.iter_mut().enumerate() {
        if block_view && index + 1 == parameter_count {
            continue;
        }
        let syn::FnArg::Typed(argument) = argument else {
            return Err(syn::Error::new(
                argument.span(),
                "component functions have no receiver",
            ));
        };
        let syn::Pat::Ident(pattern) = argument.pat.as_ref() else {
            return Err(syn::Error::new(
                argument.pat.span(),
                "use named state/component parameters",
            ));
        };
        let field = &pattern.ident;
        let mut ty = (*argument.ty).clone();
        super::normalize_type(&mut ty);
        let context = matches!(&ty, Type::Reference(reference) if matches!(reference.elem.as_ref(), Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Cx")));
        if context {
            if !argument.attrs.is_empty() {
                return Err(syn::Error::new(
                    argument.span(),
                    "context parameters cannot declare state",
                ));
            }
            contexts.push(quote!(#field: #ty));
            bound_arguments.push(quote!(self.__actor_context));
            descriptor_arguments.push(quote!(#field));
            continue;
        }
        let attrs = std::mem::take(&mut argument.attrs);
        fields.push(quote!(#(#attrs)* pub #field: #ty));
        let is_state = matches!(&ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "State" || segment.ident == "Task" || segment.ident == "Resource"));
        bound_arguments.push(if is_state {
            quote!(self.#field.handle())
        } else {
            quote!(self.#field.__descriptor())
        });
        descriptor_arguments.push(quote!(self.#field));
    }
    let mut declaration: syn::ItemStruct = syn::parse2(
        quote!(#[allow(non_camel_case_types)] #visibility struct #name { #(#fields,)* }),
    )?;
    let namespace = super::actor_namespace(attributes, name.to_string())?;
    let declaration = super::expand_declaration(&mut declaration, &namespace, true)?;
    let mut implementation: syn::ItemImpl = syn::parse2(quote!(impl #name {
        fn view(&self) #output { #name(#(#bound_arguments),*) }
    }))?;
    super::infer_bound_methods(&mut implementation);
    let implementation = super::expand_implementation_kind(&mut implementation, true, true, root)?;
    Ok(quote! {
        #[allow(non_snake_case)]
        #function
        #declaration
        #implementation
        impl #name {
            /// Render the component using its data-only scoped state descriptors.
            pub fn view(self, #(#contexts),*) #output {
                ::blockui::__component_view(#name(#(#descriptor_arguments),*), self.__actor_identity,
                    ::blockui::Key::named(concat!(module_path!(), "::", stringify!(#name))))
            }
        }
    })
}
