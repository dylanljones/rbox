// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

const ENUMS: [&str; 6] = [
    // "Analyzed",
    "HistoryType",
    "HotCueBanklistType",
    "MyTagType",
    "PlaylistType",
    "RelatedTracksType",
    "SamplerType",
];

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

    // Extract field information
    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            fields.named.iter().collect::<Vec<_>>()
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

    // Find the field to use for __repr__ (defaults to first field)
    let repr_field = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .find(|f| {
                    f.attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("repr_field"))
                })
                .or_else(|| fields.named.first())
                .and_then(|f| f.ident.as_ref())
        } else {
            None
        }
    } else {
        None
    };

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
        quote! {
            fn __repr__(&self) -> PyResult<String> {
                Ok(format!(concat!(#py_name, "(", stringify!(#field), ": {})"), self.#field))
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

            pub fn __iter__(&self, py: Python) -> PyResult<Py<PyStrIter>> {
                let keys = Self::field_names();
                Py::new(py, PyStrIter::new(keys))
            }

            pub fn __getitem__(&self, py: Python, key: &str) -> PyResult<PyObject> {
                match key {
                    #(#getter_arms)*
                    _ => Err(pyo3::exceptions::PyKeyError::new_err(format!("Key '{}' not found", key))),
                }
            }

            pub fn __setitem__(&mut self, py: Python, key: &str, value: PyObject) -> PyResult<()> {
                match key {
                    #(#setter_arms)*
                    _ => Err(pyo3::exceptions::PyKeyError::new_err(format!("Key '{}' not found", key))),
                }
            }

            pub fn keys(&self, py: Python) -> PyResult<Py<PyStrIter>> {
                let keys = Self::field_names();
                Py::new(py, PyStrIter::new(keys))
            }

            pub fn values(&self, py: Python) -> PyResult<Py<PyObjectIter>> {
                let values: Vec<PyObject> = Self::field_names()
                    .iter()
                    .filter_map(|key| self.__getitem__(py, key).ok())
                    .collect();
                Py::new(py, PyObjectIter::new(values))
            }

            pub fn items(&self, py: Python) -> PyResult<Py<PyItemsIter>> {
                let items: Vec<(String, PyObject)> = Self::field_names()
                    .iter()
                    .filter_map(|key| {
                        self.__getitem__(py, key)
                            .ok()
                            .map(|val| (key.clone(), val))
                    })
                    .collect();
                Py::new(py, PyItemsIter::new(items))
            }

            pub fn get(&self, py: Python, key: &str, default: PyObject) -> PyResult<PyObject> {
                match self.__getitem__(py, key) {
                    Ok(value) => Ok(value),
                    Err(_) => Ok(default),
                }
            }

            #[pyo3(name = "update")]
            pub fn update_py(&mut self, py: Python, data: std::collections::HashMap<String, PyObject>) -> PyResult<()> {
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
