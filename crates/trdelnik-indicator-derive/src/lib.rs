//! Proc-macro for deriving indicator parameter handling.
//!
//! # Usage
//!
//! ```ignore
//! use trdelnik_indicator_derive::indicator;
//!
//! #[indicator]
//! #[derive(Debug, Clone)]
//! pub struct Sma {
//!     #[param]
//!     period: usize,
//!
//!     // Non-param fields are internal state
//!     buffer: RingBuffer,
//! }
//!
//! // If all params have defaults, Default is auto-generated:
//! #[indicator]
//! #[derive(Debug, Clone)]
//! pub struct Bollinger {
//!     #[param(default = 20)]
//!     period: usize,
//!     #[param(default = 2.0)]
//!     std_dev_mult: f64,
//!     buffer: RingBuffer,
//! }
//!
//! impl Bollinger {
//!     // new() must always take all parameters
//!     pub fn new(period: usize, std_dev_mult: f64) -> Self { ... }
//! }
//! // Default impl is auto-generated: Self::new(20, 2.0)
//! ```
//!
//! This generates:
//! - `Hash` and `Eq` implementations based on `#[param]` fields
//! - `IndicatorParams` trait with `param_defs()` and `from_params()`
//! - `Default` impl if all `#[param]` fields have defaults
//! - Registration with the global indicator registry via `inventory`
//!
//! You still need to implement the `Indicator` trait manually for the
//! indicator logic (reset, next, warmup_period).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Ident, Meta, Type, Visibility};

/// Information about a parameter field
struct ParamField {
    name: Ident,
    ty: Type,
    default: Option<Expr>,
}

/// Parse #[param] or #[param(default = ...)] attribute
fn parse_param_attr(field: &syn::Field) -> Option<ParamField> {
    let name = field.ident.clone()?;
    let ty = field.ty.clone();

    for attr in &field.attrs {
        if attr.path().is_ident("param") {
            let mut default = None;

            // Check for #[param(default = expr)]
            if let Meta::List(list) = &attr.meta {
                let _ = list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("default") {
                        let value: Expr = meta.value()?.parse()?;
                        default = Some(value);
                    }
                    Ok(())
                });
            }

            return Some(ParamField { name, ty, default });
        }
    }
    None
}

/// Get the ParamType variant for a Rust type
fn param_type_for(ty: &Type) -> TokenStream2 {
    let ty_str = quote!(#ty).to_string();
    match ty_str.as_str() {
        "usize" => quote!(crate::ParamType::Usize),
        "f64" => quote!(crate::ParamType::F64),
        "i64" => quote!(crate::ParamType::I64),
        "bool" => quote!(crate::ParamType::Bool),
        _ => quote!(crate::ParamType::Usize), // fallback
    }
}

/// Get the ParamValue variant for a Rust type
fn param_value_variant(ty: &Type) -> Ident {
    let ty_str = quote!(#ty).to_string();
    match ty_str.as_str() {
        "usize" => format_ident!("Usize"),
        "f64" => format_ident!("F64"),
        "i64" => format_ident!("I64"),
        "bool" => format_ident!("Bool"),
        _ => format_ident!("Usize"),
    }
}

/// The `#[indicator]` attribute macro.
///
/// Generates `Hash`, `Eq`, `IndicatorParams`, and registry registration
/// based on fields marked with `#[param]`.
///
/// The macro expects that `new()` takes all `#[param]` fields as arguments.
/// For default values, implement the `Default` trait separately.
#[proc_macro_attribute]
pub fn indicator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let expanded = match expand_indicator(input) {
        Ok(tokens) => tokens,
        Err(err) => return err.to_compile_error().into(),
    };

    expanded.into()
}

/// Derive macro for generating `OutputToValue` implementation for indicator output structs.
///
/// This macro generates the boilerplate code needed to convert a multi-output
/// indicator's result struct into a `Value::Struct`.
///
/// # Requirements
///
/// - All fields must be `pub` and of type `f64`
/// - The struct must derive `Copy`
///
/// # Example
///
/// ```ignore
/// use trdelnik_indicator_derive::IndicatorOutput;
///
/// #[derive(Debug, Clone, Copy, PartialEq, IndicatorOutput)]
/// pub struct BollingerValue {
///     pub upper: f64,
///     pub middle: f64,
///     pub lower: f64,
/// }
/// ```
///
/// This generates:
///
/// ```ignore
/// impl trdelnik_core::OutputToValue for BollingerValue {
///     fn to_value(opt: Option<Self>) -> trdelnik_core::Value {
///         match opt {
///             Some(v) => {
///                 let mut fields = std::collections::BTreeMap::new();
///                 fields.insert("upper".to_string(), trdelnik_core::Value::number(v.upper));
///                 fields.insert("middle".to_string(), trdelnik_core::Value::number(v.middle));
///                 fields.insert("lower".to_string(), trdelnik_core::Value::number(v.lower));
///                 trdelnik_core::Value::structure(fields)
///             }
///             None => trdelnik_core::Value::none_struct(),
///         }
///     }
/// }
/// ```
#[proc_macro_derive(IndicatorOutput)]
pub fn derive_indicator_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match expand_indicator_output(input) {
        Ok(tokens) => tokens,
        Err(err) => return err.to_compile_error().into(),
    };

    expanded.into()
}

fn expand_indicator_output(input: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;

    // Extract fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "IndicatorOutput requires named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "IndicatorOutput can only be derived for structs",
            ))
        }
    };

    // Collect public f64 fields
    let mut field_inserts = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new_spanned(field, "expected named field")
        })?;

        // Check if field is public
        let is_pub = matches!(&field.vis, Visibility::Public(_));
        if !is_pub {
            continue; // Skip non-public fields
        }

        let field_name_str = field_name.to_string();

        field_inserts.push(quote! {
            fields.insert(
                #field_name_str.to_string(),
                trdelnik_core::Value::number(v.#field_name)
            );
        });
    }

    if field_inserts.is_empty() {
        return Err(syn::Error::new_spanned(
            &input,
            "IndicatorOutput requires at least one public field",
        ));
    }

    Ok(quote! {
        impl trdelnik_core::OutputToValue for #struct_name {
            fn to_value(opt: Option<Self>) -> trdelnik_core::Value {
                match opt {
                    Some(v) => {
                        let mut fields = std::collections::BTreeMap::new();
                        #(#field_inserts)*
                        trdelnik_core::Value::structure(fields)
                    }
                    None => trdelnik_core::Value::none_struct(),
                }
            }
        }
    })
}

fn expand_indicator(input: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;

    // Extract fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => return Err(syn::Error::new_spanned(&input, "expected named fields")),
        },
        _ => return Err(syn::Error::new_spanned(&input, "expected struct")),
    };

    // Separate param fields from state fields
    let param_fields: Vec<_> = fields.iter().filter_map(parse_param_attr).collect();
    let all_fields: Vec<_> = fields.iter().collect();

    // Generate Hash impl (only param fields)
    let hash_fields: Vec<_> = param_fields
        .iter()
        .map(|p| {
            let name = &p.name;
            let ty = &p.ty;
            let ty_str = quote!(#ty).to_string();
            if ty_str == "f64" {
                quote!(self.#name.to_bits().hash(state);)
            } else {
                quote!(self.#name.hash(state);)
            }
        })
        .collect();

    // Generate PartialEq impl (only param fields)
    let eq_fields: Vec<_> = param_fields
        .iter()
        .map(|p| {
            let name = &p.name;
            quote!(self.#name == other.#name)
        })
        .collect();

    // Handle empty param_fields case for eq
    let eq_body = if eq_fields.is_empty() {
        quote!(true)
    } else {
        quote!(#(#eq_fields)&&*)
    };

    // Generate param_defs
    let param_defs: Vec<_> = param_fields
        .iter()
        .map(|p| {
            let name_str = p.name.to_string();
            let param_type = param_type_for(&p.ty);
            let default_expr = match &p.default {
                Some(expr) => {
                    let variant = param_value_variant(&p.ty);
                    quote!(Some(crate::ParamValue::#variant(#expr)))
                }
                None => quote!(None),
            };

            quote! {
                crate::ParamDef {
                    name: #name_str,
                    param_type: #param_type,
                    default: #default_expr,
                }
            }
        })
        .collect();

    let param_count = param_fields.len();

    // Generate from_params match arms
    let from_params_extractions: Vec<_> = param_fields
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = &p.name;
            let variant = param_value_variant(&p.ty);
            quote! {
                let #name = match &params[#i] {
                    crate::ParamValue::#variant(v) => *v,
                    _ => return Err(format!(
                        "expected {} for parameter '{}'",
                        stringify!(#variant),
                        stringify!(#name)
                    )),
                };
            }
        })
        .collect();

    // Generate constructor call - always uses new()
    let param_names: Vec<_> = param_fields.iter().map(|p| &p.name).collect();

    // Check if all params have defaults - if so, generate Default impl
    // (but only if there are params - otherwise new() takes no args and user can impl Default manually)
    let all_have_defaults = !param_fields.is_empty() && param_fields.iter().all(|p| p.default.is_some());
    let default_impl = if all_have_defaults {
        let default_values: Vec<_> = param_fields.iter().map(|p| p.default.as_ref().unwrap()).collect();
        quote! {
            impl Default for #struct_name {
                fn default() -> Self {
                    Self::new(#(#default_values),*)
                }
            }
        }
    } else {
        quote! {}
    };

    // Reconstruct the struct definition with original fields (without #[param] attrs)
    let field_defs: Vec<_> = all_fields
        .iter()
        .map(|f| {
            let attrs: Vec<_> = f
                .attrs
                .iter()
                .filter(|a| !a.path().is_ident("param"))
                .collect();
            let vis = &f.vis;
            let name = &f.ident;
            let ty = &f.ty;
            quote!(#(#attrs)* #vis #name: #ty)
        })
        .collect();

    Ok(quote! {
        #(#attrs)*
        #vis struct #struct_name #generics {
            #(#field_defs,)*
        }

        impl std::hash::Hash for #struct_name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                #(#hash_fields)*
            }
        }

        impl PartialEq for #struct_name {
            fn eq(&self, other: &Self) -> bool {
                #eq_body
            }
        }

        impl Eq for #struct_name {}

        impl crate::IndicatorParams for #struct_name {
            fn param_defs() -> &'static [crate::ParamDef] {
                static PARAMS: &[crate::ParamDef] = &[
                    #(#param_defs,)*
                ];
                PARAMS
            }

            fn from_params(params: &[crate::ParamValue]) -> Result<Self, String> {
                if params.len() != #param_count {
                    return Err(format!(
                        "expected {} parameters, got {}",
                        #param_count,
                        params.len()
                    ));
                }

                #(#from_params_extractions)*

                Ok(Self::new(#(#param_names),*))
            }
        }

        #default_impl

        // Register with inventory for compile-time collection
        inventory::submit! {
            crate::IndicatorMeta::of::<#struct_name>()
        }
    })
}
