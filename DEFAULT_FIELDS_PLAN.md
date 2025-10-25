# Default Fields Implementation Plan

## Overview

Add support for `#[default(value)]` attribute that allows fields to have default values in the builder. These fields are not part of the typestate progression and can be optionally overridden using mutable setter methods.

## Feature Requirements

### Input Example
```rust
#[derive(Builder)]
struct Config {
    #[default(42)]
    foo: u8,
    baz: u8
}
```

### Generated Output

**Key differences from current implementation:**

1. **Builder struct** - Default fields are concrete, not generic:
   ```rust
   // Before: ConfigBuilder<Foo, Baz>
   // After:  ConfigBuilder<Baz>  // Only non-default fields are generic

   ConfigBuilder<Baz> {
       foo: u8,      // Concrete type, not generic
       baz: Baz      // Still generic
   }
   ```

2. **Constructor** - Default fields initialized with their default values:
   ```rust
   impl ConfigBuilder<()> {
       pub fn new() -> Self {
           Self {
               foo: 42,    // Default value
               baz: ()
           }
       }
   }
   ```

3. **Default field setters** - Mutable, non-consuming:
   ```rust
   pub fn with_foo(&mut self, foo: u8) -> &mut Self {
       self.foo = foo;
       self
   }
   ```

4. **Regular field setters** - Consuming (unchanged):
   ```rust
   pub fn with_baz(self, baz: u8) -> ConfigBuilder<u8> {
       ConfigBuilder { foo: self.foo, baz }
   }
   ```

5. **Build method** - Only requires non-default fields to be set:
   ```rust
   impl ConfigBuilder<u8> {  // Only non-default generics
       pub fn build(self) -> Config {
           Config { foo: self.foo, baz: self.baz }
       }
   }
   ```

## Implementation Steps

### 1. Extend Attribute Parsing

**Add function to detect `#[default(...)]` attribute:**

```rust
use syn::{Expr, Lit};

/// Check if a field has #[default(...)] attribute and extract the value
fn extract_default_value(field: &Field) -> Option<Expr> {
    for attr in &field.attrs {
        if attr.path().is_ident("default") {
            // Parse the attribute arguments
            if let Ok(expr) = attr.parse_args::<Expr>() {
                return Some(expr);
            }
        }
    }
    None
}

/// Check if a field has a default value
fn has_default_attr(field: &Field) -> bool {
    extract_default_value(field).is_some()
}
```

### 2. Categorize Fields

Create a helper to classify fields:

```rust
enum FieldCategory {
    Default {
        value: Expr,        // The default value expression
        ty: Type,           // The concrete type
    },
    Regular {
        ty: Type,           // The type (may need smart wrapping)
    },
}

struct CategorizedField {
    name: Ident,
    category: FieldCategory,
    wrapper: WrapperType,   // For Arc/Box smart wrapping
}

fn categorize_field(field: &Field) -> CategorizedField {
    let name = field.ident.as_ref().unwrap().clone();
    let ty = field.ty.clone();
    let wrapper = analyze_field_type(&ty);

    if let Some(default_value) = extract_default_value(field) {
        CategorizedField {
            name,
            category: FieldCategory::Default { value: default_value, ty },
            wrapper,
        }
    } else {
        CategorizedField {
            name,
            category: FieldCategory::Regular { ty },
            wrapper,
        }
    }
}
```

### 3. Update Builder Struct Generation

**Only create generic parameters for non-default fields:**

```rust
fn generate_builder_struct(builder_name: &Ident, fields: &[Field]) -> TokenStream {
    let categorized: Vec<_> = fields.iter().map(categorize_field).collect();

    // Only non-default fields get generic parameters
    let type_params: Vec<_> = categorized
        .iter()
        .filter_map(|f| match &f.category {
            FieldCategory::Regular { .. } => {
                Some(field_name_to_type_param(&f.name))
            }
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
```

### 4. Update Constructor Generation

**Initialize default fields with their values:**

```rust
fn generate_constructor(builder_name: &Ident, fields: &[Field]) -> TokenStream {
    let categorized: Vec<_> = fields.iter().map(categorize_field).collect();

    // Type parameters: () for regular fields, nothing for default fields
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
```

### 5. Generate Two Types of Setters

**For default fields - mutable, non-consuming:**

```rust
// For field with #[default(42)]
pub fn with_foo(&mut self, foo: u8) -> &mut Self {
    self.foo = foo;
    self
}
```

**For regular fields - consuming (current behavior):**

```rust
// For field without #[default]
pub fn with_baz(self, baz: u8) -> ConfigBuilder<u8> {
    ConfigBuilder { foo: self.foo, baz }
}
```

**Implementation:**

```rust
fn generate_builder_methods(builder_name: &Ident, fields: &[Field]) -> TokenStream {
    let categorized: Vec<_> = fields.iter().map(categorize_field).collect();

    // Extract just the regular (non-default) fields for generics
    let regular_fields: Vec<_> = categorized
        .iter()
        .filter(|f| matches!(f.category, FieldCategory::Regular { .. }))
        .collect();

    let type_params: Vec<_> = regular_fields
        .iter()
        .map(|f| field_name_to_type_param(&f.name))
        .collect();

    let methods: Vec<_> = categorized
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let field_name = &field.name;
            let method_name = Ident::new(&format!("with_{}", field_name), field_name.span());

            match &field.category {
                FieldCategory::Default { ty, .. } => {
                    // Mutable setter for default fields
                    match &field.wrapper {
                        WrapperType::Arc(bounds) => {
                            let trait_bounds = generate_trait_bounds(bounds);
                            quote! {
                                pub fn #method_name(&mut self, #field_name: impl #trait_bounds) -> &mut Self {
                                    self.#field_name = std::sync::Arc::new(#field_name);
                                    self
                                }
                            }
                        }
                        WrapperType::Box(bounds) => {
                            let trait_bounds = generate_trait_bounds(bounds);
                            quote! {
                                pub fn #method_name(&mut self, #field_name: impl #trait_bounds) -> &mut Self {
                                    self.#field_name = Box::new(#field_name);
                                    self
                                }
                            }
                        }
                        WrapperType::None => {
                            quote! {
                                pub fn #method_name(&mut self, #field_name: #ty) -> &mut Self {
                                    self.#field_name = #field_name;
                                    self
                                }
                            }
                        }
                    }
                }

                FieldCategory::Regular { ty } => {
                    // Consuming setter for regular fields (existing logic)
                    // Find position in regular_fields to determine return type
                    let regular_idx = regular_fields.iter().position(|f| f.name == field_name).unwrap();

                    let return_type_params: Vec<_> = type_params
                        .iter()
                        .enumerate()
                        .map(|(i, param)| {
                            if i == regular_idx {
                                quote! { #ty }
                            } else {
                                quote! { #param }
                            }
                        })
                        .collect();

                    // Build field assignments
                    let field_assignments: Vec<_> = categorized
                        .iter()
                        .map(|f| {
                            let fname = &f.name;
                            if fname == field_name {
                                match &field.wrapper {
                                    WrapperType::Arc(_) => quote! { #fname: std::sync::Arc::new(#fname) },
                                    WrapperType::Box(_) => quote! { #fname: Box::new(#fname) },
                                    WrapperType::None => quote! { #fname },
                                }
                            } else {
                                quote! { #fname: self.#fname }
                            }
                        })
                        .collect();

                    // Generate parameter type (with smart wrapper support)
                    let param_type = match &field.wrapper {
                        WrapperType::Arc(bounds) | WrapperType::Box(bounds) => {
                            let trait_bounds = generate_trait_bounds(bounds);
                            quote! { impl #trait_bounds }
                        }
                        WrapperType::None => quote! { #ty },
                    };

                    quote! {
                        pub fn #method_name(self, #field_name: #param_type) -> #builder_name<#(#return_type_params),*> {
                            #builder_name {
                                #(#field_assignments),*
                            }
                        }
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
```

### 6. Update build() Method

**Only require non-default fields to be concrete:**

```rust
fn generate_build_method(
    struct_name: &Ident,
    builder_name: &Ident,
    fields: &[Field],
) -> TokenStream {
    let categorized: Vec<_> = fields.iter().map(categorize_field).collect();

    // Only non-default fields in the type parameters
    let concrete_types: Vec<_> = categorized
        .iter()
        .filter_map(|f| match &f.category {
            FieldCategory::Regular { ty } => Some(ty),
            FieldCategory::Default { .. } => None,
        })
        .collect();

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
```

### 7. Update Type Alias Generation

**Incomplete type alias should only reference non-default fields:**

```rust
fn generate_type_aliases(
    struct_name: &Ident,
    builder_name: &Ident,
    fields: &[Field],
) -> TokenStream {
    let categorized: Vec<_> = fields.iter().map(categorize_field).collect();

    // Check if there are incomplete fields (considering only non-default fields)
    let has_incomplete = categorized
        .iter()
        .any(|f| matches!(f.category, FieldCategory::Regular { .. }) && has_incomplete_attr_by_name(&f.name, fields));

    if has_incomplete {
        let type_params_incomplete: Vec<_> = categorized
            .iter()
            .filter_map(|f| match &f.category {
                FieldCategory::Regular { ty } => {
                    if has_incomplete_attr_by_name(&f.name, fields) {
                        Some(quote! { () })
                    } else {
                        Some(quote! { #ty })
                    }
                }
                FieldCategory::Default { .. } => None,  // Default fields don't appear in type params
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
```

### 8. Update Macro Entry Point

**Register the new `default` attribute:**

```rust
#[proc_macro_derive(Builder, attributes(incomplete, default))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    // ... existing code
}
```

## Edge Cases to Handle

### 1. Default + Incomplete
What happens if a field has both attributes?

**Decision:** `#[default]` takes precedence. The field has a default value and is not part of typestate.

```rust
#[default(42)]
#[incomplete]
foo: u8
```

Generates:
```rust
ConfigBuilder {  // No generic for foo
    foo: u8,
}
```

### 2. Default + Smart Wrapper
Default values with Arc/Box:

```rust
#[default(Arc::new(ConsoleLogger))]
logger: Arc<dyn Logger>
```

Should work as-is - the default value expression is used directly.

Setter still supports smart wrapping:
```rust
builder.with_logger(MyLogger);  // Auto-wraps
```

### 3. Complex Default Expressions

```rust
#[default(vec![1, 2, 3])]
items: Vec<i32>

#[default(String::from("default"))]
name: String

#[default(MyStruct { field: 42 })]
config: MyStruct
```

All should work - we parse the expression and use it directly.

### 4. Chaining with Mutable Setters

Users can chain mutable setters:

```rust
ConfigBuilder::new()
    .with_foo(10)    // &mut Self
    .with_foo(20)    // &mut Self, can override
    .with_baz(30)    // Consumes, returns ConfigBuilder<u8>
    .build()
```

## Testing Strategy

### Test 1: Basic Default Field
```rust
#[derive(Builder)]
struct Config {
    #[default(42)]
    timeout: u64,
    url: String,
}

#[test]
fn test_default_field() {
    let config = ConfigBuilder::new()
        .with_url("https://example.com".to_string())
        .build();

    assert_eq!(config.timeout, 42);
    assert_eq!(config.url, "https://example.com");
}
```

### Test 2: Overriding Default
```rust
#[test]
fn test_override_default() {
    let config = ConfigBuilder::new()
        .with_timeout(100)
        .with_url("https://example.com".to_string())
        .build();

    assert_eq!(config.timeout, 100);
}
```

### Test 3: Multiple Defaults
```rust
#[derive(Builder)]
struct Config {
    #[default(42)]
    timeout: u64,
    #[default(3)]
    retries: u32,
    url: String,
}
```

### Test 4: Default with Smart Wrapper
```rust
struct DefaultLogger;
impl LoggerTrait for DefaultLogger { /* ... */ }

#[derive(Builder)]
struct App {
    #[default(Arc::new(DefaultLogger))]
    logger: Arc<dyn LoggerTrait>,
    name: String,
}

#[test]
fn test_default_with_smart_wrapper() {
    let app = AppBuilder::new()
        .with_logger(CustomLogger)  // Can override, auto-wraps
        .with_name("App".to_string())
        .build();
}
```

### Test 5: All Fields Default
```rust
#[derive(Builder)]
struct Config {
    #[default(42)]
    foo: u64,
    #[default(String::from("default"))]
    bar: String,
}

// Builder has no generics!
impl ConfigBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn build(self) -> Config { /* ... */ }
}
```

### Test 6: Chaining Mutable Setters
```rust
#[test]
fn test_chaining_mutable_setters() {
    let config = ConfigBuilder::new()
        .with_timeout(100)
        .with_timeout(200)  // Can override
        .with_url("test".to_string())
        .build();

    assert_eq!(config.timeout, 200);
}
```

## Documentation Updates

### README.md

Add new section:

```markdown
### Default Field Values

Fields can have default values using the `#[default(...)]` attribute. These fields:
- Are initialized with the default value in `new()`
- Can be optionally overridden with mutable setters
- Don't affect typestate progression

\```rust
#[derive(Builder)]
struct ServerConfig {
    #[default(8080)]
    port: u16,
    #[default(4)]
    workers: usize,
    host: String,  // Required field
}

let config = ServerConfigBuilder::new()
    .with_host("localhost".to_string())
    .build();

assert_eq!(config.port, 8080);  // Default value

// Override defaults
let config = ServerConfigBuilder::new()
    .with_port(3000)  // Override default
    .with_host("localhost".to_string())
    .build();

assert_eq!(config.port, 3000);
\```

**Mutable setters:**

Default fields use mutable setters (`&mut self`) instead of consuming setters, allowing multiple overrides:

\```rust
let mut builder = ServerConfigBuilder::new();
builder.with_port(3000);
builder.with_port(4000);  // Can override again
let config = builder.with_host("localhost".to_string()).build();

assert_eq!(config.port, 4000);
\```
```

## Implementation Phases

### Phase 1: Core Functionality
1. Parse `#[default(...)]` attribute
2. Categorize fields (default vs regular)
3. Update builder struct generation
4. Update constructor generation
5. Generate mutable setters for default fields
6. Update build() method

### Phase 2: Integration
7. Update type alias generation
8. Ensure smart wrapper works with defaults
9. Handle edge cases (default + incomplete)

### Phase 3: Testing & Documentation
10. Write comprehensive tests
11. Update README
12. Test all edge cases

## Potential Issues

### Issue 1: Mutable vs Consuming API

**Problem:** Mixing `&mut self` and `self` methods can be confusing.

**Solution:**
- Clear documentation
- Consistent naming (all use `with_*`)
- Examples showing both patterns

### Issue 2: Default Expression Parsing

**Problem:** Complex expressions might not parse correctly.

**Solution:**
- Use `syn::Expr` which handles all expressions
- Document supported expression types
- Provide clear error messages for parse failures

### Issue 3: Type Parameter Ordering

**Problem:** Removing some type parameters while keeping others might affect ordering.

**Solution:**
- Only iterate over non-default fields when creating generics
- Maintain consistent iteration order throughout

## Recommendation

Implement all three phases as they build on each other:
1. Core functionality provides the basic feature
2. Integration ensures it works with existing features
3. Testing validates everything works correctly

This feature significantly improves ergonomics by reducing boilerplate for commonly-configured fields while maintaining type safety for required fields.
