// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Python insertion constructors that preserve native defaults and private state.

use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, Fields, Ident, ItemStruct, Token, Visibility};

pub(super) fn expand(item: &ItemStruct) -> syn::Result<syn::File> {
    let Fields::Named(fields) = &item.fields else {
        return Err(syn::Error::new_spanned(
            item,
            "PyConstructor requires named fields",
        ));
    };
    if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "PyConstructor does not support generic models",
        ));
    }
    let fields: Vec<_> = fields
        .named
        .iter()
        .filter(|field| matches!(field.vis, Visibility::Public(_)))
        .collect();
    let mut required = Vec::<Ident>::new();
    let mut constructor = None::<Expr>;
    for attr in item
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("py_constructor"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("required") {
                let content;
                syn::parenthesized!(content in meta.input);
                for name in Punctuated::<Ident, Token![,]>::parse_terminated(&content)? {
                    if required.contains(&name) {
                        return Err(syn::Error::new_spanned(name, "duplicate required field"));
                    }
                    required.push(name);
                }
            } else if meta.path.is_ident("constructor") {
                if constructor.is_some() {
                    return Err(meta.error("duplicate constructor"));
                }
                constructor = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error("expected required or constructor"));
            }
            Ok(())
        })?;
    }
    let mut arguments = Vec::new();
    for name in &required {
        let field = fields
            .iter()
            .find(|field| field.ident.as_ref() == Some(name))
            .ok_or_else(|| {
                syn::Error::new_spanned(name, "expected an unrestricted public field")
            })?;
        if name == "kwargs" {
            return Err(syn::Error::new_spanned(
                name,
                "kwargs is reserved by PyConstructor",
            ));
        }
        let ty = &field.ty;
        arguments.push(quote!(#name: #ty));
    }
    let constructor = constructor.unwrap_or_else(|| syn::parse_quote!(Self::new(#(#required),*)));
    let setters = fields
        .iter()
        .filter(|field| !required.contains(field.ident.as_ref().unwrap()))
        .map(|field| {
            let name = &field.ident;
            let key = name
                .as_ref()
                .unwrap()
                .to_string()
                .trim_start_matches("r#")
                .to_owned();
            let ty = &field.ty;
            quote!(#key => model.#name = value.extract::<#ty>()?,)
        });
    let name = &item.ident;
    syn::parse2(quote! {
        #[::pyo3::pymethods]
        impl #name {
            #[new]
            #[pyo3(signature = (#(#required,)* **kwargs))]
            fn __rbox_py_new(
                #(#arguments,)*
                kwargs: ::std::option::Option<&::pyo3::Bound<'_, ::pyo3::types::PyDict>>,
            ) -> ::pyo3::PyResult<Self> {
                use ::pyo3::types::{PyAnyMethods as _, PyDictMethods as _};
                let mut model = #constructor;
                if let Some(kwargs) = kwargs {
                    for (key, value) in kwargs.iter() {
                        let key = key.extract::<::std::string::String>()?;
                        match key.as_str() {
                            #(#setters)*
                            _ => return Err(::pyo3::exceptions::PyTypeError::new_err(
                                format!("{}() got an unexpected keyword argument '{}'", stringify!(#name), key)
                            )),
                        }
                    }
                }
                Ok(model)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::expand;
    use syn::parse_quote;

    #[test]
    fn invalid_declarations_have_diagnostics() {
        for (item, expected) in [
            (
                parse_quote!(
                    struct Tuple(pub i64);
                ),
                "named fields",
            ),
            (
                parse_quote!(
                    struct Generic<T> {
                        pub value: T,
                    }
                ),
                "generic models",
            ),
            (
                parse_quote!(
                    #[py_constructor(required(hidden))]
                    struct Model {
                        hidden: i64,
                    }
                ),
                "public field",
            ),
            (
                parse_quote!(
                    #[py_constructor(required(missing))]
                    struct Model {
                        pub id: i64,
                    }
                ),
                "public field",
            ),
            (
                parse_quote!(
                    #[py_constructor(required(id, id))]
                    struct Model {
                        pub id: i64,
                    }
                ),
                "duplicate required",
            ),
            (
                parse_quote!(
                    #[py_constructor(constructor = Self::new(), constructor = Self::new())]
                    struct Model {}
                ),
                "duplicate constructor",
            ),
            (
                parse_quote!(
                    #[py_constructor(unknown)]
                    struct Model {}
                ),
                "expected required or constructor",
            ),
        ] {
            assert!(expand(&item).unwrap_err().to_string().contains(expected));
        }
    }
}
