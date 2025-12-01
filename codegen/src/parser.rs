// Author: Dylan Jones
// Date:   2025-11-22

use anyhow::Result;
use quote::ToTokens;
use std::path::Path;
use syn::File;

use super::models::*;

fn parse_function(f: &syn::ItemFn, is_method: bool) -> FunctionInfo {
    FunctionInfo {
        name: f.sig.ident.to_string(),
        vis: vis_info(&f.vis),
        attrs: attrs_info(&f.attrs),
        doc: collect_doc(&f.attrs),
        generics: generics_info(&f.sig.generics),
        args: fn_args(&f.sig.inputs),
        ret: match &f.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(parse_type(ty)),
        },
        is_async: f.sig.asyncness.is_some(),
        is_unsafe: f.sig.unsafety.is_some(),
        is_const: f.sig.constness.is_some(),
        abi: f
            .sig
            .abi
            .as_ref()
            .map(|abi| abi.to_token_stream().to_string()),
        is_method,
    }
}

fn parse_struct(s: &syn::ItemStruct) -> StructInfo {
    let fields = match &s.fields {
        syn::Fields::Named(named) => named
            .named
            .iter()
            .map(|f| FieldInfo {
                name: f.ident.as_ref().map(|i| i.to_string()),
                vis: vis_info(&f.vis),
                ty: parse_type(&f.ty),
                attrs: attrs_info(&f.attrs),
                doc: collect_doc(&f.attrs),
            })
            .collect(),
        syn::Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| FieldInfo {
                name: Some(i.to_string()), // tuple field index, for convenience
                vis: vis_info(&f.vis),
                ty: parse_type(&f.ty),
                attrs: attrs_info(&f.attrs),
                doc: collect_doc(&f.attrs),
            })
            .collect(),
        syn::Fields::Unit => Vec::new(),
    };

    StructInfo {
        name: s.ident.to_string(),
        vis: vis_info(&s.vis),
        attrs: attrs_info(&s.attrs),
        doc: collect_doc(&s.attrs),
        generics: generics_info(&s.generics),
        fields,
    }
}

fn parse_impl(imp: &syn::ItemImpl) -> ImplInfo {
    let self_ty = type_to_str(&imp.self_ty);

    let trait_path = if let Some((_, path, _)) = &imp.trait_ {
        Some(path.to_token_stream().to_string())
    } else {
        None
    };

    let mut methods = Vec::new();
    for item in &imp.items {
        if let syn::ImplItem::Fn(m) = item {
            // synthesize a temporary ItemFn to reuse extractor
            let dummy_block: syn::Block = syn::parse_quote!({ /* stub */ });
            let f = syn::ItemFn {
                attrs: m.attrs.clone(),
                vis: m.vis.clone(),
                sig: m.sig.clone(),
                block: Box::new(dummy_block),
            };
            methods.push(parse_function(&f, true));
        }
    }

    ImplInfo {
        attrs: attrs_info(&imp.attrs),
        generics: generics_info(&imp.generics),
        vis: vis_info(&syn::Visibility::Inherited),
        self_ty,
        trait_path,
        doc: collect_doc(&imp.attrs),
        methods,
    }
}

pub fn parse_rust_source(src: &str) -> Result<ModuleAst> {
    let file: File = syn::parse_file(src)?;

    let mut ast = ModuleAst::default();
    for item in file.items {
        match item {
            syn::Item::Fn(f) => ast.functions.push(parse_function(&f, false)),
            syn::Item::Struct(s) => ast.structs.push(parse_struct(&s)),
            syn::Item::Impl(imp) => ast.impls.push(parse_impl(&imp)),
            _ => {}
        }
    }
    Ok(ast)
}

pub fn parse_rust_file<P: AsRef<Path>>(path: P) -> Result<ModuleAst> {
    let src = std::fs::read_to_string(path)?;
    parse_rust_source(&src)
}

// ---------- Helpers ----------

fn vis_info(vis: &syn::Visibility) -> VisibilityInfo {
    let raw = vis.to_token_stream().to_string();
    VisibilityInfo { raw }
}

fn attrs_info(attrs: &[syn::Attribute]) -> Vec<AttrInfo> {
    attrs
        .iter()
        .map(|a| AttrInfo {
            raw: a.to_token_stream().to_string(),
            path: a.path().to_token_stream().to_string(),
        })
        .filter(|a| a.path != "doc")
        .collect()
}

fn generics_info(g: &syn::Generics) -> GenericInfo {
    let params = if g.params.is_empty() {
        String::new()
    } else {
        let mut buf = String::from("<");
        let pieces: Vec<String> = g
            .params
            .iter()
            .map(|p| match p {
                syn::GenericParam::Type(t) => t.to_token_stream().to_string(),
                syn::GenericParam::Lifetime(l) => l.to_token_stream().to_string(),
                syn::GenericParam::Const(c) => c.to_token_stream().to_string(),
            })
            .collect();
        buf.push_str(&pieces.join(", "));
        buf.push('>');
        buf
    };

    let where_clause = g
        .where_clause
        .as_ref()
        .map(|w| w.to_token_stream().to_string())
        .unwrap_or_default();

    GenericInfo {
        params,
        where_clause,
    }
}

fn type_to_str(t: &syn::Type) -> String {
    t.to_token_stream().to_string()
}

fn parse_type_str(s: String) -> TypeInfo {
    let raw = s.clone();
    if s.contains("<") {
        // Split inner
        let mut pieces = s.split("<");
        // Collect all pieces except the first one, which is the type name
        let name = pieces.next().unwrap().trim().to_string();
        let inner = pieces.collect::<Vec<&str>>().join("<");
        // Remove the last '>' from inner
        let inner = inner.trim_end_matches('>').trim().to_string();
        // Split by commas, but be careful of nested generics
        let mut children = Vec::new();
        let mut depth = 0;
        let mut current = String::new();
        for c in inner.chars() {
            if c == '<' {
                depth += 1;
            } else if c == '>' {
                depth -= 1;
            } else if c == ',' && depth == 0 {
                children.push(parse_type_str(current.trim().to_string()));
                current.clear();
                continue;
            }
            current.push(c);
        }
        if !current.is_empty() {
            children.push(parse_type_str(current.trim().to_string()));
        }
        TypeInfo {
            name,
            children,
            raw,
        }
    } else {
        TypeInfo {
            name: s.clone(),
            children: vec![],
            raw,
        }
    }
}

fn parse_type(t: &syn::Type) -> TypeInfo {
    let s = type_to_str(t);
    parse_type_str(s)
}

fn collect_doc(attrs: &[syn::Attribute]) -> Doc {
    let mut lines = Vec::new();
    for a in attrs {
        if a.path().is_ident("doc") {
            match &a.meta {
                syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }),
                    ..
                }) => lines.push(s.value().trim().to_string()),
                syn::Meta::NameValue(syn::MetaNameValue { value, .. }) => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = value
                    {
                        lines.push(s.value().trim().to_string());
                    }
                }
                _ => {}
            }
        }
    }
    Doc { lines }
}

fn fn_args(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Vec<FunctionArg> {
    inputs
        .iter()
        .map(|a| match a {
            syn::FnArg::Receiver(recv) => FunctionArg {
                name: "self".into(),
                ty: None,
                is_self: true,
                attrs: attrs_info(&recv.attrs),
            },
            syn::FnArg::Typed(p) => FunctionArg {
                name: pat_to_name(&p.pat),
                ty: Some(parse_type(&p.ty)),
                is_self: false,
                attrs: attrs_info(&p.attrs),
            },
        })
        .collect()
}

fn pat_to_name(p: &syn::Pat) -> String {
    match p {
        syn::Pat::Ident(i) => i.ident.to_string(),
        // be conservative; you can improve this to show actual patterns
        _ => "_".into(),
    }
}
