// Author: Dylan Jones
// Date:   2025-11-22

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
    pub name: String,       // "self" or pattern name or "_"
    pub ty: Option<String>, // None for receiver; Some("<type>") for typed arg
    pub is_self: bool,
    pub attrs: Vec<AttrInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: Option<String>, // None for tuple fields
    pub vis: VisibilityInfo,
    pub ty: String,
    pub attrs: Vec<AttrInfo>,
    pub doc: Doc,
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

/// A Rust function.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub vis: VisibilityInfo,
    pub attrs: Vec<AttrInfo>,
    pub doc: Doc,
    pub generics: GenericInfo,
    pub args: Vec<FunctionArg>,
    pub ret: Option<String>, // None => no return type
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_const: bool,
    pub abi: Option<String>, // e.g. "C"
    pub is_method: bool,     // true if from impl
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

/// The AST of a Rust module.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleAst {
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
    pub impls: Vec<ImplInfo>,
}
