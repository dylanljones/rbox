// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Generate plain JavaScript model objects without exposing native private state.

use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Attribute, Field, Fields, Ident, ItemStruct, Token, Visibility};

use crate::is_option_type;

#[derive(Default)]
struct Options {
    output: Option<Ident>,
    patch: Option<Ident>,
    readonly: Vec<Ident>,
    input: Option<Ident>,
    required: Vec<Ident>,
    constructor: Option<Ident>,
}

fn docs(field: &Field) -> Vec<&Attribute> {
    field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .collect()
}

pub(super) fn expand(item: &ItemStruct) -> syn::Result<syn::File> {
    let Fields::Named(fields) = &item.fields else {
        return Err(syn::Error::new_spanned(
            item,
            "NapiPublic requires named fields",
        ));
    };
    if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "NapiPublic does not support generic models",
        ));
    }
    let fields: Vec<_> = fields
        .named
        .iter()
        .filter(|field| matches!(field.vis, Visibility::Public(_)))
        .collect();
    for field in &fields {
        if is_option_type(&field.ty) {
            return Err(syn::Error::new_spanned(field, "NapiPublic requires nonoptional public fields; nullable patch semantics must be defined separately"));
        }
    }
    let mut options = Options::default();
    for attr in item
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("napi_public"))
    {
        attr.parse_nested_meta(|meta| {
            let slot = if meta.path.is_ident("output") {
                Some(&mut options.output)
            } else if meta.path.is_ident("input") {
                Some(&mut options.input)
            } else if meta.path.is_ident("patch") {
                Some(&mut options.patch)
            } else if meta.path.is_ident("constructor") {
                Some(&mut options.constructor)
            } else {
                None
            };
            if let Some(slot) = slot {
                if slot.is_some() {
                    return Err(meta.error("duplicate option"));
                }
                *slot = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("readonly") || meta.path.is_ident("required") {
                let content;
                syn::parenthesized!(content in meta.input);
                let names = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
                let target = if meta.path.is_ident("readonly") {
                    &mut options.readonly
                } else {
                    &mut options.required
                };
                for name in names {
                    if target.contains(&name) {
                        return Err(syn::Error::new_spanned(name, "duplicate field"));
                    }
                    target.push(name);
                }
            } else {
                return Err(
                    meta.error("expected output, patch, readonly, input, required, or constructor")
                );
            }
            Ok(())
        })?;
    }
    for name in options.readonly.iter().chain(&options.required) {
        if !fields
            .iter()
            .any(|field| field.ident.as_ref() == Some(name))
        {
            return Err(syn::Error::new_spanned(
                name,
                "expected an unrestricted public field",
            ));
        }
    }
    let model = &item.ident;
    let vis = &item.vis;
    let js_name = model.to_string();
    if let Some(object) = options.output {
        if options.input.is_some() || options.constructor.is_some() || !options.required.is_empty()
        {
            return Err(syn::Error::new_spanned(
                item,
                "output mode cannot specify input, constructor, or required",
            ));
        }
        if options.patch.is_none() && !options.readonly.is_empty() {
            return Err(syn::Error::new_spanned(item, "readonly requires patch"));
        }
        let declarations = fields.iter().map(|field| {
            let name = &field.ident;
            let ty = &field.ty;
            let docs = docs(field);
            quote!(#(#docs)* pub #name: #ty)
        });
        let assignments = fields.iter().map(|field| {
            let name = &field.ident;
            quote!(#name: model.#name)
        });
        let patch = options.patch.map(|patch| {
            let mutable: Vec<_> = fields.iter().filter(|field| !options.readonly.contains(field.ident.as_ref().unwrap())).collect();
            let declarations = mutable.iter().map(|field| {
                let name = &field.ident;
                let ty = &field.ty;
                let docs = docs(field);
                quote!(#(#docs)* pub #name: ::std::option::Option<#ty>)
            });
            let assignments = mutable.iter().map(|field| {
                let name = &field.ident;
                quote!(if let Some(value) = self.#name { model.#name = value; })
            });
            quote! {
                #[doc = "Partial changes. Omitted or undefined fields retain their current values; null is rejected."]
                #[::napi_derive::napi(object, object_to_js = false)]
                #[derive(Default)]
                #vis struct #patch { #(#declarations,)* }

                impl #patch {
                    #[doc = "Apply supplied public fields without touching private state."]
                    pub fn apply_to(self, model: &mut #model) { #(#assignments)* }
                }
            }
        });
        syn::parse2(quote! {
            #[doc = "Detached public values; editing this object does not persist changes."]
            #[::napi_derive::napi(object, object_from_js = false, js_name = #js_name)]
            #vis struct #object { #(#declarations,)* }

            impl From<#model> for #object {
                fn from(model: #model) -> Self { Self { #(#assignments,)* } }
            }

            impl ::napi::bindgen_prelude::ToNapiValue for #model {
                unsafe fn to_napi_value(env: ::napi::sys::napi_env, value: Self) -> ::napi::Result<::napi::sys::napi_value> {
                    // SAFETY: forwards the caller's environment unchanged to napi's object converter.
                    unsafe { <#object as ::napi::bindgen_prelude::ToNapiValue>::to_napi_value(env, value.into()) }
                }
            }

            impl ::napi::bindgen_prelude::TypeName for #model {
                fn type_name() -> &'static str { #js_name }
                fn value_type() -> ::napi::ValueType { ::napi::ValueType::Object }
            }
            #patch
        })
    } else if let Some(input) = options.input {
        if options.patch.is_some() || !options.readonly.is_empty() {
            return Err(syn::Error::new_spanned(
                item,
                "input mode cannot specify patch or readonly",
            ));
        }
        let constructor = options
            .constructor
            .ok_or_else(|| syn::Error::new_spanned(item, "input mode requires constructor"))?;
        let declarations = fields.iter().map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            let docs = docs(field);
            if options.required.contains(name) {
                quote!(#(#docs)* pub #name: #ty)
            } else {
                quote!(#(#docs)* pub #name: ::std::option::Option<#ty>)
            }
        });
        let assignments = fields
            .iter()
            .filter(|field| !options.required.contains(field.ident.as_ref().unwrap()))
            .map(|field| {
                let name = &field.ident;
                quote!(if let Some(value) = input.#name { model.#name = value; })
            });
        let required = &options.required;
        syn::parse2(quote! {
            #[doc = "Insertion input. Optional fields override the model constructor's defaults."]
            #[::napi_derive::napi(object, object_to_js = false, js_name = #js_name)]
            #vis struct #input { #(#declarations,)* }

            impl From<#input> for #model {
                fn from(input: #input) -> Self {
                    let mut model = Self::#constructor(#(input.#required),*);
                    #(#assignments)*
                    model
                }
            }
        })
    } else {
        Err(syn::Error::new_spanned(
            item,
            "NapiPublic requires output or input",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn struct_fields(file: &syn::File, name: &str) -> Vec<String> {
        file.items
            .iter()
            .find_map(|item| match item {
                syn::Item::Struct(item) if item.ident == name => Some(
                    item.fields
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap().to_string())
                        .collect(),
                ),
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn objects_and_patches_exclude_native_state_and_readonly_fields() {
        let generated = expand(&parse_quote! {
            #[napi_public(output = Data, patch = Changes, readonly(id))]
            pub struct Model {
                pub id: i64,
                pub title: String,
                pub(crate) internal: NativeOnly,
                raw: NativeOnly,
            }
        })
        .unwrap();
        assert_eq!(struct_fields(&generated, "Data"), ["id", "title"]);
        assert_eq!(struct_fields(&generated, "Changes"), ["title"]);
    }

    #[test]
    fn inputs_require_constructor_arguments_and_make_metadata_optional() {
        let generated = expand(&parse_quote! {
            #[napi_public(input = Input, required(path), constructor = new)]
            pub struct Model {
                pub path: String,
                pub title: String,
                creation: NativeOnly,
            }
        })
        .unwrap();
        assert_eq!(struct_fields(&generated, "Input"), ["path", "title"]);
        let syn::Item::Struct(input) = &generated.items[0] else {
            panic!()
        };
        let fields: Vec<_> = input.fields.iter().collect();
        assert!(!is_option_type(&fields[0].ty));
        assert!(is_option_type(&fields[1].ty));
    }

    #[test]
    fn invalid_options_and_ambiguous_nullable_fields_have_diagnostics() {
        let cases: Vec<(ItemStruct, &str)> = vec![
            (
                parse_quote!(
                    struct Tuple(pub i64);
                ),
                "requires named fields",
            ),
            (
                parse_quote!(
                    struct Model<T> {
                        pub value: T,
                    }
                ),
                "generic models",
            ),
            (
                parse_quote!(
                    #[napi_public(output = Data)]
                    struct Model {
                        pub value: Option<String>,
                    }
                ),
                "nonoptional public fields",
            ),
            (
                parse_quote!(
                    #[napi_public(input = Input)]
                    struct Model {
                        pub title: String,
                    }
                ),
                "requires constructor",
            ),
            (
                parse_quote!(
                    #[napi_public(output = Data, input = Input)]
                    struct Model {
                        pub id: i64,
                    }
                ),
                "output mode cannot specify",
            ),
            (
                parse_quote!(
                    #[napi_public(output = Data, patch = Changes, readonly(raw))]
                    struct Model {
                        pub id: i64,
                        raw: NativeOnly,
                    }
                ),
                "unrestricted public field",
            ),
            (
                parse_quote!(
                    #[napi_public(output = Data, patch = Changes, readonly(missing))]
                    struct Model {
                        pub id: i64,
                    }
                ),
                "unrestricted public field",
            ),
            (
                parse_quote!(
                    #[napi_public(output = Data, patch = Changes, readonly(id, id))]
                    struct Model {
                        pub id: i64,
                    }
                ),
                "duplicate field",
            ),
            (
                parse_quote!(
                    #[napi_public(output = Data, output = Other)]
                    struct Model {
                        pub id: i64,
                    }
                ),
                "duplicate option",
            ),
            (
                parse_quote!(
                    #[napi_public(typo = Data)]
                    struct Model {
                        pub id: i64,
                    }
                ),
                "expected output",
            ),
        ];
        for (item, message) in cases {
            assert!(
                expand(&item).unwrap_err().to_string().contains(message),
                "{message}"
            );
        }
    }
}
