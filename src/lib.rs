use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, Data, Fields, Field, Ident, Type, Expr,
    GenericArgument, PathArguments, TypeParamBound, TypeTraitObject,
};

#[proc_macro_derive(Builder, attributes(incomplete, default))]
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

/// Extract default value from #[default(...)] attribute
fn extract_default_value(field: &Field) -> Option<Expr> {
    for attr in &field.attrs {
        if attr.path().is_ident("default") {
            if let Ok(expr) = attr.parse_args::<Expr>() {
                return Some(expr);
            }
        }
    }
    None
}

/// Check if a field has a #[default(...)] attribute
fn has_default_attr(field: &Field) -> bool {
    extract_default_value(field).is_some()
}

/// Check if any field has the #[incomplete] attribute (excluding default fields)
fn has_any_incomplete_fields(fields: &[Field]) -> bool {
    fields.iter().any(|f| {
        // Only count as incomplete if it has #[incomplete] but NOT #[default]
        has_incomplete_attr(f) && !has_default_attr(f)
    })
}

// ============================================================================
// Field Categorization
// ============================================================================

/// Categories for fields based on their attributes
enum FieldCategory {
    Default {
        value: Expr,  // The default value expression
        ty: Type,     // The concrete type
    },
    Regular {
        ty: Type,     // The type (may need smart wrapping)
    },
}

/// A categorized field with its name, category, and wrapper type
struct CategorizedField {
    name: Ident,
    category: FieldCategory,
    wrapper: WrapperType,
    has_incomplete: bool,  // Track if field has #[incomplete] attribute
}

/// Categorize a field as either default or regular
fn categorize_field(field: &Field) -> CategorizedField {
    let name = field.ident.as_ref().unwrap().clone();
    let ty = field.ty.clone();
    let wrapper = analyze_field_type(&ty);
    let has_incomplete = has_incomplete_attr(field);

    // #[default] takes precedence over #[incomplete]
    if let Some(default_value) = extract_default_value(field) {
        CategorizedField {
            name,
            category: FieldCategory::Default { value: default_value, ty },
            wrapper,
            has_incomplete: false,  // Default fields ignore #[incomplete]
        }
    } else {
        CategorizedField {
            name,
            category: FieldCategory::Regular { ty },
            wrapper,
            has_incomplete,
        }
    }
}

/// Categorize all fields
fn categorize_fields(fields: &[Field]) -> Vec<CategorizedField> {
    fields.iter().map(categorize_field).collect()
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

// ============================================================================
// Type Analysis for Smart Wrappers
// ============================================================================

/// Represents different wrapper types for fields
enum WrapperType {
    None,  // Regular field, no wrapping
    Arc(Vec<TypeParamBound>),  // Arc<dyn Trait> - bounds are the trait bounds
    Box(Vec<TypeParamBound>),  // Box<dyn Trait> - bounds are the trait bounds
}

/// Check if a type is Arc<dyn Trait> and extract the trait bounds
fn is_arc_dyn_trait(ty: &Type) -> Option<Vec<TypeParamBound>> {
    if let Type::Path(type_path) = ty {
        // Check if the path has segments and the last one is "Arc"
        let last_segment = type_path.path.segments.last()?;

        if last_segment.ident != "Arc" {
            return None;
        }

        // Get generic arguments
        if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
            if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                // Check if inner type is TraitObject (has dyn keyword)
                if let Type::TraitObject(trait_obj) = inner_ty {
                    return Some(trait_obj.bounds.iter().cloned().collect());
                }
            }
        }
    }

    None
}

/// Check if a type is Box<dyn Trait> and extract the trait bounds
fn is_box_dyn_trait(ty: &Type) -> Option<Vec<TypeParamBound>> {
    if let Type::Path(type_path) = ty {
        // Check if the path has segments and the last one is "Box"
        let last_segment = type_path.path.segments.last()?;

        if last_segment.ident != "Box" {
            return None;
        }

        // Get generic arguments
        if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
            if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                // Check if inner type is TraitObject (has dyn keyword)
                if let Type::TraitObject(trait_obj) = inner_ty {
                    return Some(trait_obj.bounds.iter().cloned().collect());
                }
            }
        }
    }

    None
}

/// Analyze a field type to determine if it needs smart wrapping
fn analyze_field_type(ty: &Type) -> WrapperType {
    if let Some(bounds) = is_arc_dyn_trait(ty) {
        return WrapperType::Arc(bounds);
    }

    if let Some(bounds) = is_box_dyn_trait(ty) {
        return WrapperType::Box(bounds);
    }

    WrapperType::None
}

// ============================================================================
// Code Generation Functions
// ============================================================================

/// Generate the builder struct definition
fn generate_builder_struct(builder_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
    let categorized = categorize_fields(fields);

    // Only non-default fields get generic parameters
    let type_params: Vec<_> = categorized
        .iter()
        .filter_map(|f| match &f.category {
            FieldCategory::Regular { .. } => Some(field_name_to_type_param(&f.name)),
            FieldCategory::Default { .. } => None,
        })
        .collect();

    let field_defs: Vec<_> = categorized
        .iter()
        .map(|f| {
            let field_name = &f.name;
            match &f.category {
                FieldCategory::Default { ty, .. } => {
                    // Default fields have concrete types
                    quote! { #field_name: #ty }
                }
                FieldCategory::Regular { .. } => {
                    // Regular fields are generic
                    let type_param = field_name_to_type_param(field_name);
                    quote! { #field_name: #type_param }
                }
            }
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
        let categorized = categorize_fields(fields);

        // Only include regular fields in type parameters (default fields are concrete)
        let type_params_incomplete: Vec<_> = categorized
            .iter()
            .filter_map(|f| match &f.category {
                FieldCategory::Regular { ty } => {
                    if f.has_incomplete {
                        Some(quote! { () })
                    } else {
                        Some(quote! { #ty })
                    }
                }
                FieldCategory::Default { .. } => None,
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
    let categorized = categorize_fields(fields);

    // Only regular fields get () type parameters
    let type_params: Vec<_> = categorized
        .iter()
        .filter_map(|f| match &f.category {
            FieldCategory::Regular { .. } => Some(quote! { () }),
            FieldCategory::Default { .. } => None,
        })
        .collect();

    let field_inits: Vec<_> = categorized
        .iter()
        .map(|f| {
            let field_name = &f.name;
            match &f.category {
                FieldCategory::Default { value, .. } => {
                    // Initialize with default value
                    quote! { #field_name: #value }
                }
                FieldCategory::Regular { .. } => {
                    // Initialize with ()
                    quote! { #field_name: () }
                }
            }
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
    let categorized = categorize_fields(fields);

    // Generate type parameters (only for regular fields)
    let type_params: Vec<_> = categorized
        .iter()
        .filter_map(|f| match &f.category {
            FieldCategory::Regular { .. } => Some(field_name_to_type_param(&f.name)),
            FieldCategory::Default { .. } => None,
        })
        .collect();

    // Split categorized fields into regular and default
    let regular_fields: Vec<_> = categorized
        .iter()
        .enumerate()
        .filter_map(|(idx, f)| match &f.category {
            FieldCategory::Regular { .. } => Some((idx, f)),
            FieldCategory::Default { .. } => None,
        })
        .collect();

    let default_fields: Vec<_> = categorized
        .iter()
        .filter_map(|f| match &f.category {
            FieldCategory::Default { .. } => Some(f),
            _ => None,
        })
        .collect();

    // Generate methods for regular fields (typestate pattern)
    let regular_methods: Vec<_> = regular_fields
        .iter()
        .enumerate()
        .map(|(regular_idx, (_, catfield))| {
            let field_name = &catfield.name;
            let field_type = match &catfield.category {
                FieldCategory::Regular { ty } => ty,
                _ => unreachable!(),
            };
            let method_name = Ident::new(&format!("with_{}", field_name), field_name.span());

            // Create return type parameters (replace the current field's generic with concrete type)
            let return_type_params: Vec<_> = type_params
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    if i == regular_idx {
                        quote! { #field_type }
                    } else {
                        quote! { #param }
                    }
                })
                .collect();

            // Create field assignments for all other fields (regular and default)
            let other_field_assignments: Vec<_> = categorized
                .iter()
                .filter(|f| f.name != *field_name)
                .map(|f| {
                    let fname = &f.name;
                    quote! { #fname: self.#fname }
                })
                .collect();

            match &catfield.wrapper {
                WrapperType::Arc(bounds) => {
                    let has_lifetime = bounds.iter().any(|b| matches!(b, TypeParamBound::Lifetime(_)));
                    let trait_bounds = if has_lifetime {
                        quote! { #(#bounds)+* }
                    } else {
                        quote! { #(#bounds)+* + 'static }
                    };

                    quote! {
                        pub fn #method_name(
                            self,
                            #field_name: impl #trait_bounds
                        ) -> #builder_name<#(#return_type_params),*> {
                            #builder_name {
                                #field_name: std::sync::Arc::new(#field_name),
                                #(#other_field_assignments),*
                            }
                        }
                    }
                }

                WrapperType::Box(bounds) => {
                    let has_lifetime = bounds.iter().any(|b| matches!(b, TypeParamBound::Lifetime(_)));
                    let trait_bounds = if has_lifetime {
                        quote! { #(#bounds)+* }
                    } else {
                        quote! { #(#bounds)+* + 'static }
                    };

                    quote! {
                        pub fn #method_name(
                            self,
                            #field_name: impl #trait_bounds
                        ) -> #builder_name<#(#return_type_params),*> {
                            #builder_name {
                                #field_name: Box::new(#field_name),
                                #(#other_field_assignments),*
                            }
                        }
                    }
                }

                WrapperType::None => {
                    quote! {
                        pub fn #method_name(self, #field_name: #field_type) -> #builder_name<#(#return_type_params),*> {
                            #builder_name {
                                #field_name,
                                #(#other_field_assignments),*
                            }
                        }
                    }
                }
            }
        })
        .collect();

    // Generate mutable consuming setters for default fields
    let default_methods: Vec<_> = default_fields
        .iter()
        .map(|catfield| {
            let field_name = &catfield.name;
            let field_type = match &catfield.category {
                FieldCategory::Default { ty, .. } => ty,
                _ => unreachable!(),
            };
            let method_name = Ident::new(&format!("with_{}", field_name), field_name.span());

            match &catfield.wrapper {
                WrapperType::Arc(bounds) => {
                    let has_lifetime = bounds.iter().any(|b| matches!(b, TypeParamBound::Lifetime(_)));
                    let trait_bounds = if has_lifetime {
                        quote! { #(#bounds)+* }
                    } else {
                        quote! { #(#bounds)+* + 'static }
                    };

                    quote! {
                        pub fn #method_name(
                            mut self,
                            #field_name: impl #trait_bounds
                        ) -> Self {
                            self.#field_name = std::sync::Arc::new(#field_name);
                            self
                        }
                    }
                }

                WrapperType::Box(bounds) => {
                    let has_lifetime = bounds.iter().any(|b| matches!(b, TypeParamBound::Lifetime(_)));
                    let trait_bounds = if has_lifetime {
                        quote! { #(#bounds)+* }
                    } else {
                        quote! { #(#bounds)+* + 'static }
                    };

                    quote! {
                        pub fn #method_name(
                            mut self,
                            #field_name: impl #trait_bounds
                        ) -> Self {
                            self.#field_name = Box::new(#field_name);
                            self
                        }
                    }
                }

                WrapperType::None => {
                    quote! {
                        pub fn #method_name(mut self, #field_name: #field_type) -> Self {
                            self.#field_name = #field_name;
                            self
                        }
                    }
                }
            }
        })
        .collect();

    quote! {
        impl<#(#type_params),*> #builder_name<#(#type_params),*> {
            #(#regular_methods)*
            #(#default_methods)*
        }
    }
}

/// Generate build() method for fully concrete builder
fn generate_build_method(
    struct_name: &Ident,
    builder_name: &Ident,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let categorized = categorize_fields(fields);

    // Only regular fields are type parameters (default fields are already concrete)
    let concrete_types: Vec<_> = categorized
        .iter()
        .filter_map(|f| match &f.category {
            FieldCategory::Regular { ty } => Some(ty),
            FieldCategory::Default { .. } => None,
        })
        .collect();

    // All fields are included in the final struct
    let field_names: Vec<_> = categorized.iter().map(|f| &f.name).collect();

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
