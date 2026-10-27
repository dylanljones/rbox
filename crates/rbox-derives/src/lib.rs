// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Procedural derives for Rbox Python bindings.
//!
//! Generates Python mapping methods for named-field models. Generated iterators
//! use the runtime helpers in `rbox-core`, whose `pyo3` feature must be enabled
//! by the consuming crate.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, Visibility};

const ENUMS: [&str; 6] = [
    // "Analyzed",
    "HistoryType",
    "HotCueBanklistType",
    "MyTagType",
    "PlaylistType",
    "RelatedTracksType",
    "SamplerType",
];

/// Implement Python mapping methods for a named-field Python class.
///
/// Only unrestricted `pub` fields participate in the mapping and its representation.
/// Private and restricted fields (such as `pub(crate)`) are ignored, including any
/// `repr_field` annotation on those fields. Hidden field types need no Python conversions.
/// This does not control properties generated separately by `pyclass`.
///
/// The consuming crate must depend on `rbox-core` with its `pyo3` feature
/// enabled. Iterator helpers are resolved through `::rbox_core::python`, so
/// they do not need to be imported into the model module.
#[proc_macro_derive(PyMutableMapping, attributes(py_name, repr_field))]
pub fn derive_py_mutable_mapping(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Get the Python class name from #[py_name] attribute or use the struct name
    let py_name = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("py_name") {
                attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            // Remove "Py" prefix if present
            let name_str = name.to_string();
            if name_str.starts_with("Py") {
                name_str[2..].to_string()
            } else {
                name_str
            }
        });

    // Use the same public fields for every generated mapping operation.
    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .filter(|field| matches!(field.vis, Visibility::Public(_)))
                .collect::<Vec<_>>()
        } else {
            panic!("PyMutableMapping only works with named fields");
        }
    } else {
        panic!("PyMutableMapping only works with structs");
    };

    let field_count = fields.len();

    // Generate field names array
    let field_names = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap().to_string();
        quote! { #name.to_string() }
    });

    // Representation must not expose hidden fields either.
    let repr_field = fields
        .iter()
        .find(|f| {
            f.attrs
                .iter()
                .any(|attr| attr.path().is_ident("repr_field"))
        })
        .or_else(|| fields.first())
        .and_then(|f| f.ident.as_ref());

    // Generate getter match arms
    let getter_arms = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &f.ty;

        // Check if it's an Option type or Py<PyDateTime>
        let conversion = if is_option_type(field_type) {
            quote! {
                self.#field_name.clone().into_pyobject(py)?.into()
            }
        } else if is_py_datetime(field_type) {
            quote! {
                self.#field_name.clone_ref(py).into_py(py)
            }
        } else if is_enum_type(field_type) {
            quote! {
                (self.#field_name as i32).into_pyobject(py)?.into()
            }
        } else {
            quote! {
                self.#field_name.clone().into_pyobject(py)?.into()
            }
        };

        quote! {
            #field_name_str => Ok(#conversion),
        }
    });

    // Generate setter match arms
    let setter_arms = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &f.ty;

        let extraction = if is_py_datetime(field_type) {
            quote! {
                value.extract::<Py<PyDateTime>>(py).unwrap()
            }
        } else if is_enum_type(field_type) {
            quote! {
                {
                    let raw: i32 = value.extract(py)?;
                    <#field_type as std::convert::TryFrom<i32>>::try_from(raw).unwrap()
                }
            }
        } else {
            quote! {
                value.extract(py)?
            }
        };

        quote! {
            #field_name_str => {
                self.#field_name = #extraction;
                Ok(())
            },
        }
    });

    let repr_impl = if let Some(field) = repr_field {
        // Optional fields (for example, an unset insertion sequence) lack Display.
        let format = if fields
            .iter()
            .any(|f| f.ident.as_ref() == Some(&field) && is_option_type(&f.ty))
        {
            ": {:?})"
        } else {
            ": {})"
        };
        quote! {
            fn __repr__(&self) -> PyResult<String> {
                Ok(format!(concat!(#py_name, "(", stringify!(#field), #format), self.#field))
            }
        }
    } else {
        quote! {
            fn __repr__(&self) -> PyResult<String> {
                Ok(format!(#py_name))
            }
        }
    };

    let expanded = quote! {
        impl #name {
            fn field_names() -> Vec<String> {
                vec![#(#field_names),*]
            }
        }

        #[pymethods]
        impl #name {
            fn __len__(&self) -> usize {
                #field_count
            }

            pub fn __contains__(&self, key: &str) -> bool {
                Self::field_names().contains(&key.to_string())
            }

            pub fn __iter__(&self, py: Python) -> PyResult<Py<::rbox_core::python::PyStrIter>> {
                let keys = Self::field_names();
                Py::new(py, ::rbox_core::python::PyStrIter::new(keys))
            }

            pub fn __getitem__(&self, py: Python, key: &str) -> PyResult<Py<PyAny>> {
                match key {
                    #(#getter_arms)*
                    _ => Err(pyo3::exceptions::PyKeyError::new_err(format!("Key '{}' not found", key))),
                }
            }

            pub fn __setitem__(&mut self, py: Python, key: &str, value: Py<PyAny>) -> PyResult<()> {
                match key {
                    #(#setter_arms)*
                    _ => Err(pyo3::exceptions::PyKeyError::new_err(format!("Key '{}' not found", key))),
                }
            }

            pub fn keys(&self, py: Python) -> PyResult<Py<::rbox_core::python::PyStrIter>> {
                let keys = Self::field_names();
                Py::new(py, ::rbox_core::python::PyStrIter::new(keys))
            }

            pub fn values(&self, py: Python) -> PyResult<Py<::rbox_core::python::PyObjectIter>> {
                let values: Vec<Py<PyAny>> = Self::field_names()
                    .iter()
                    .filter_map(|key| self.__getitem__(py, key).ok())
                    .collect();
                Py::new(py, ::rbox_core::python::PyObjectIter::new(values))
            }

            pub fn items(&self, py: Python) -> PyResult<Py<::rbox_core::python::PyItemsIter>> {
                let items: Vec<(String, Py<PyAny>)> = Self::field_names()
                    .iter()
                    .filter_map(|key| {
                        self.__getitem__(py, key)
                            .ok()
                            .map(|val| (key.clone(), val))
                    })
                    .collect();
                Py::new(py, ::rbox_core::python::PyItemsIter::new(items))
            }

            pub fn get(&self, py: Python, key: &str, default: Py<PyAny>) -> PyResult<Py<PyAny>> {
                match self.__getitem__(py, key) {
                    Ok(value) => Ok(value),
                    Err(_) => Ok(default),
                }
            }

            #[pyo3(name = "update")]
            pub fn update_py(&mut self, py: Python, data: std::collections::HashMap<String, Py<PyAny>>) -> PyResult<()> {
                for (key, value) in data {
                    self.__setitem__(py, &key, value)?;
                }
                Ok(())
            }

            pub fn to_dict(&self, py: Python) -> PyResult<Py<pyo3::types::PyDict>> {
                let dict = pyo3::types::PyDict::new(py);
                for key in Self::field_names() {
                    if let Ok(value) = self.__getitem__(py, &key) {
                        dict.set_item(&key, value)?;
                    }
                }
                Ok(dict.into())
            }

            #repr_impl
        }
    };

    TokenStream::from(expanded)
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn is_enum_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        let path_str = quote!(#type_path).to_string();
        for enum_name in ENUMS {
            if path_str.contains(enum_name) {
                return true;
            }
        }
    }
    false
}

fn is_py_datetime(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        let path_str = quote!(#type_path).to_string();
        return path_str.contains("Py < PyDateTime >");
    }
    false
}
