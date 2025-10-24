use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Ident};

#[proc_macro_derive(Builder, attributes(incomplete))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract struct name and fields
    let struct_name = &input.ident;
    let fields = match extract_fields(&input) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error().into(),
    };

    // Create builder name
    let builder_name = create_builder_name(struct_name);

    // Generate all components
    let builder_struct = generate_builder_struct(&builder_name, &fields);
    let type_aliases = generate_type_aliases(struct_name, &builder_name, &fields);
    let constructor = generate_constructor(&builder_name, &fields);
    let builder_methods = generate_builder_methods(&builder_name, &fields);
    let build_method = generate_build_method(struct_name, &builder_name, &fields);

    let expanded = quote! {
        #builder_struct
        #type_aliases
        #constructor
        #builder_methods
        #build_method
    };

    TokenStream::from(expanded)
}

/// Extract named fields from struct
fn extract_fields(input: &DeriveInput) -> syn::Result<Vec<Field>> {
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => Ok(fields.named.iter().cloned().collect()),
            _ => Err(syn::Error::new_spanned(
                &data_struct.fields,
                "Builder can only be derived for structs with named fields",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            input,
            "Builder can only be derived for structs",
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

/// Create builder name (e.g., "Config" -> "ConfigBuilder")
fn create_builder_name(struct_name: &Ident) -> Ident {
    let name = format!("{}Builder", struct_name);
    Ident::new(&name, struct_name.span())
}

/// Create type alias name (e.g., "Config" + "Complete" -> "ConfigComplete")
fn create_type_alias_name(struct_name: &Ident, suffix: &str) -> Ident {
    let name = format!("{}{}", struct_name, suffix);
    Ident::new(&name, struct_name.span())
}

/// Generate the builder struct definition
fn generate_builder_struct(builder_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
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
        struct #builder_name<#(#type_params),*> {
            #(#field_defs),*
        }
    }
}

/// Generate type aliases (conditionally Incomplete only)
fn generate_type_aliases(
    struct_name: &Ident,
    builder_name: &Ident,
    fields: &[Field],
) -> proc_macro2::TokenStream {
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
        quote! {
            pub type #incomplete_name = #builder_name<#(#type_params_incomplete),*>;
        }
    } else {
        quote! {}
    }
}

/// Generate the new() constructor
fn generate_constructor(builder_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
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
        impl #builder_name<#(#type_params),*> {
            pub fn new() -> Self {
                Self {
                    #(#field_inits),*
                }
            }
        }
    }
}

/// Generate builder methods for each field
fn generate_builder_methods(builder_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
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
                pub fn #method_name(self, #field_name: #field_type) -> #builder_name<#(#return_type_params),*> {
                    #builder_name {
                        #(#field_assignments),*
                    }
                }
            }
        })
        .collect();

    quote! {
        impl<#(#type_params),*> #builder_name<#(#type_params),*> {
            #(#methods)*
        }
    }
}

/// Generate build() method for fully concrete builder
fn generate_build_method(
    struct_name: &Ident,
    builder_name: &Ident,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let concrete_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();

    quote! {
        impl #builder_name<#(#concrete_types),*> {
            pub fn build(self) -> #struct_name {
                #struct_name {
                    #(#field_names: self.#field_names),*
                }
            }
        }
    }
}
