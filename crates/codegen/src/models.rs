// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Structured descriptions of Rust syntax used by the source parser.
//!
//! Captures documentation, attributes, types, fields, function signatures, and
//! implementation blocks for consumers that generate code from Rust sources.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Doc {
    pub lines: Vec<String>,
}

impl Doc {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttrInfo {
    /// The full attribute tokens, e.g. `#[pyo3(name = "foo", text)]`
    pub raw: String,
    /// The attribute path (head), e.g. `pyo3` or `doc`
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenericInfo {
    pub params: String,       // e.g. "<'a, T>"
    pub where_clause: String, // e.g. "where T: Clone"
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisibilityInfo {
    pub raw: String, // "pub", "pub(crate)", or "" (inherited)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub children: Vec<TypeInfo>,
    pub raw: String,
}

impl TypeInfo {
    pub fn to_string(&self) -> String {
        let mut out = self.name.clone();
        if !self.children.is_empty() {
            out.push_str("<");
            for (i, child) in self.children.iter().enumerate() {
                out.push_str(&child.to_string());
                if i != self.children.len() - 1 {
                    out.push_str(", ");
                }
            }
            out.push_str(">");
        }
        out.replace("& ", "&")
    }
}

impl VisibilityInfo {
    pub fn is_public(&self) -> bool {
        self.raw == "pub"
    }

    pub fn is_public_crate(&self) -> bool {
        self.raw == "pub(crate)"
    }

    pub fn is_inherited(&self) -> bool {
        self.raw.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionArg {
    pub name: String,         // "self" or pattern name or "_"
    pub ty: Option<TypeInfo>, // None for receiver; Some("<type>") for typed arg
    pub is_self: bool,
    pub attrs: Vec<AttrInfo>,
}

impl FunctionArg {
    pub fn find_attrs(&self, s: &str) -> Vec<&AttrInfo> {
        self.attrs.iter().filter(|a| a.raw.contains(s)).collect()
    }

    pub fn attrs_contains(&self, s: &str) -> bool {
        self.attrs.iter().any(|a| a.raw.contains(s))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: Option<String>, // None for tuple fields
    pub vis: VisibilityInfo,
    pub ty: TypeInfo,
    pub attrs: Vec<AttrInfo>,
    pub doc: Doc,
}

impl FieldInfo {
    pub fn find_attrs(&self, s: &str) -> Vec<&AttrInfo> {
        self.attrs.iter().filter(|a| a.raw.contains(s)).collect()
    }

    pub fn attrs_contains(&self, s: &str) -> bool {
        self.attrs.iter().any(|a| a.raw.contains(s))
    }
}

/// A Rust struct.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub vis: VisibilityInfo,
    pub attrs: Vec<AttrInfo>,
    pub doc: Doc,
    pub generics: GenericInfo,
    pub fields: Vec<FieldInfo>, // works for both named and tuple structs
}

impl StructInfo {
    pub fn find_attrs(&self, s: &str) -> Vec<&AttrInfo> {
        self.attrs.iter().filter(|a| a.raw.contains(s)).collect()
    }

    pub fn attrs_contains(&self, s: &str) -> bool {
        self.attrs.iter().any(|a| a.raw.contains(s))
    }
}

/// A Rust function.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub vis: VisibilityInfo,
    pub attrs: Vec<AttrInfo>,
    pub doc: Doc,
    pub generics: GenericInfo,
    pub args: Vec<FunctionArg>,
    pub ret: Option<TypeInfo>, // None => no return type
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_const: bool,
    pub abi: Option<String>, // e.g. "C"
    pub is_method: bool,     // true if from impl
}

impl FunctionInfo {
    pub fn find_attrs(&self, s: &str) -> Vec<&AttrInfo> {
        self.attrs.iter().filter(|a| a.raw.contains(s)).collect()
    }

    pub fn attrs_contains(&self, s: &str) -> bool {
        self.attrs.iter().any(|a| a.raw.contains(s))
    }
}

/// A Rust `impl` block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImplInfo {
    pub attrs: Vec<AttrInfo>,
    pub generics: GenericInfo,
    pub vis: VisibilityInfo,        // usually inherited for impls
    pub self_ty: String,            // the type being implemented
    pub trait_path: Option<String>, // Some("TraitPath") for trait impls
    pub doc: Doc,
    pub methods: Vec<FunctionInfo>,
}

impl ImplInfo {
    pub fn find_attrs(&self, s: &str) -> Vec<&AttrInfo> {
        self.attrs.iter().filter(|a| a.raw.contains(s)).collect()
    }

    pub fn attrs_contains(&self, s: &str) -> bool {
        self.attrs.iter().any(|a| a.raw.contains(s))
    }
}

/// The AST of a Rust module.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleAst {
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
    pub impls: Vec<ImplInfo>,
}

impl ModuleAst {
    pub fn find_struct(&self, name: &str) -> Option<&StructInfo> {
        self.structs.iter().find(|s| s.name == name)
    }

    pub fn find_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn find_impl(&self, self_ty: &str) -> Vec<&ImplInfo> {
        self.impls.iter().filter(|i| i.self_ty == self_ty).collect()
    }

    pub fn find_struct_methods(&self, self_ty: &str) -> Vec<&FunctionInfo> {
        let mut methods = Vec::new();
        for impl_block in self.find_impl(self_ty).iter() {
            for method in impl_block.methods.iter() {
                methods.push(method);
            }
        }
        methods
    }
}
