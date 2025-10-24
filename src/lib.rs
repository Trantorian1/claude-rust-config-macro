use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Ident};

#[proc_macro_derive(Config, attributes(incomplete))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract struct name and fields
    let struct_name = &input.ident;
    let fields = match extract_fields(&input.data) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error().into(),
    };

    // Generate all components
    let generic_struct = generate_generic_struct(struct_name, &fields);
    let type_aliases = generate_type_aliases(struct_name, &fields);
    let constructor = generate_constructor(struct_name, &fields);
    let builder_methods = generate_builder_methods(struct_name, &fields);

    let expanded = quote! {
        #generic_struct
        #type_aliases
        #constructor
        #builder_methods
    };

    TokenStream::from(expanded)
}

/// Extract named fields from struct
fn extract_fields(data: &Data) -> syn::Result<Vec<Field>> {
    match data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => Ok(fields.named.iter().cloned().collect()),
            _ => Err(syn::Error::new_spanned(
                &data_struct.fields,
                "Config can only be derived for structs with named fields",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            data,
            "Config can only be derived for structs",
        )),
    }
}

/// Check if a field has the #[incomplete] attribute
fn has_incomplete_attr(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("incomplete"))
}

/// Check if any field has the #[incomplete] attribute
fn has_any_incomplete_fields(fields: &[Field]) -> bool {
    fields.iter().any(has_incomplete_attr)
}

/// Convert field name to PascalCase type parameter
fn field_name_to_type_param(name: &Ident) -> Ident {
    let name_str = name.to_string();
    let pascal_case = name_str
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect::<String>();
    Ident::new(&pascal_case, name.span())
}

/// Create type alias name (e.g., "Config" + "Complete" -> "ConfigComplete")
fn create_type_alias_name(struct_name: &Ident, suffix: &str) -> Ident {
    let name = format!("{}{}", struct_name, suffix);
    Ident::new(&name, struct_name.span())
}

/// Generate the generic struct definition
fn generate_generic_struct(struct_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
    let type_params: Vec<_> = fields
        .iter()
        .map(|f| field_name_to_type_param(f.ident.as_ref().unwrap()))
        .collect();

    let field_defs: Vec<_> = fields
        .iter()
        .zip(&type_params)
        .map(|(field, type_param)| {
            let field_name = &field.ident;
            quote! { #field_name: #type_param }
        })
        .collect();

    quote! {
        struct #struct_name<#(#type_params),*> {
            #(#field_defs),*
        }
    }
}

/// Generate type aliases (Complete and conditionally Incomplete)
fn generate_type_aliases(struct_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
    let type_params_complete: Vec<_> = fields
        .iter()
        .map(|f| &f.ty)
        .collect();

    let complete_name = create_type_alias_name(struct_name, "Complete");
    let complete_alias = quote! {
        pub type #complete_name = #struct_name<#(#type_params_complete),*>;
    };

    // Only generate incomplete if there are #[incomplete] markers
    if has_any_incomplete_fields(fields) {
        let type_params_incomplete: Vec<_> = fields
            .iter()
            .map(|f| {
                if has_incomplete_attr(f) {
                    quote! { () }
                } else {
                    let ty = &f.ty;
                    quote! { #ty }
                }
            })
            .collect();

        let incomplete_name = create_type_alias_name(struct_name, "Incomplete");
        let incomplete_alias = quote! {
            pub type #incomplete_name = #struct_name<#(#type_params_incomplete),*>;
        };

        quote! {
            #complete_alias
            #incomplete_alias
        }
    } else {
        complete_alias
    }
}

/// Generate the new() constructor
fn generate_constructor(struct_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
    let type_params: Vec<_> = fields
        .iter()
        .map(|_| quote! { () })
        .collect();

    let field_inits: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = &f.ident;
            quote! { #field_name: () }
        })
        .collect();

    quote! {
        impl #struct_name<#(#type_params),*> {
            pub fn new() -> Self {
                Self {
                    #(#field_inits),*
                }
            }
        }
    }
}

/// Generate builder methods for each field
fn generate_builder_methods(struct_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
    let type_params: Vec<_> = fields
        .iter()
        .map(|f| field_name_to_type_param(f.ident.as_ref().unwrap()))
        .collect();

    let methods: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let method_name = Ident::new(&format!("with_{}", field_name), field_name.span());

            // Create return type parameters (replace the current field's generic with concrete type)
            let return_type_params: Vec<_> = type_params
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    if i == idx {
                        quote! { #field_type }
                    } else {
                        quote! { #param }
                    }
                })
                .collect();

            // Create field assignments
            let field_assignments: Vec<_> = fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let fname = f.ident.as_ref().unwrap();
                    if i == idx {
                        quote! { #fname: #field_name }
                    } else {
                        quote! { #fname: self.#fname }
                    }
                })
                .collect();

            quote! {
                pub fn #method_name(self, #field_name: #field_type) -> #struct_name<#(#return_type_params),*> {
                    #struct_name {
                        #(#field_assignments),*
                    }
                }
            }
        })
        .collect();

    quote! {
        impl<#(#type_params),*> #struct_name<#(#type_params),*> {
            #(#methods)*
        }
    }
}
