# Smart Wrapper Feature Implementation Plan

## Overview

Add automatic wrapping support for `Arc<dyn Trait>` and `Box<dyn Trait>` fields in builder methods. Instead of requiring users to manually wrap values, the builder will accept `impl Trait` and handle wrapping automatically.

## Feature Requirements

### Current Behavior
```rust
#[derive(Builder)]
struct Config {
    handler: Arc<dyn Handler>,
}

// User must manually wrap
let config = ConfigBuilder::new()
    .with_handler(Arc::new(MyHandler::new()))
    .build();
```

### Desired Behavior
```rust
#[derive(Builder)]
struct Config {
    handler: Arc<dyn Handler>,
}

// Builder automatically wraps
let config = ConfigBuilder::new()
    .with_handler(MyHandler::new())  // Just pass the value!
    .build();
```

## Implementation Steps

### 1. Type Analysis Module

Create helper functions to analyze field types and detect wrapper patterns.

**Functions needed:**

```rust
/// Check if type is Arc<dyn Trait>
fn is_arc_dyn_trait(ty: &Type) -> Option<&Type> {
    // Parse Type::Path
    // Check if outer type is "Arc" from std::sync
    // Check if inner type has "dyn" keyword
    // Return Some(trait_type) if match, None otherwise
}

/// Check if type is Box<dyn Trait>
fn is_box_dyn_trait(ty: &Type) -> Option<&Type> {
    // Parse Type::Path
    // Check if outer type is "Box"
    // Check if inner type has "dyn" keyword
    // Return Some(trait_type) if match, None otherwise
}

/// Extract trait from dyn Trait type
fn extract_trait_from_dyn(ty: &Type) -> Option<&Type> {
    // Handle Type::TraitObject
    // Extract the main trait (first in bounds)
    // Return trait type without "dyn" keyword
}

enum FieldWrapperType {
    None,           // Regular field, no wrapping
    Arc(Type),      // Arc<dyn Trait> - trait is the Type
    Box(Type),      // Box<dyn Trait> - trait is the Type
}

fn analyze_field_type(ty: &Type) -> FieldWrapperType {
    if let Some(inner) = is_arc_dyn_trait(ty) {
        if let Some(trait_ty) = extract_trait_from_dyn(inner) {
            return FieldWrapperType::Arc(trait_ty.clone());
        }
    }

    if let Some(inner) = is_box_dyn_trait(ty) {
        if let Some(trait_ty) = extract_trait_from_dyn(inner) {
            return FieldWrapperType::Box(trait_ty.clone());
        }
    }

    FieldWrapperType::None
}
```

### 2. Update Builder Method Generation

Modify `generate_builder_methods()` to handle wrapped types differently.

**Current logic:**
```rust
pub fn with_handler(self, handler: Arc<dyn Handler>) -> ConfigBuilder<Arc<dyn Handler>, ...> {
    ConfigBuilder {
        handler,
        ...
    }
}
```

**New logic:**
```rust
pub fn with_handler(self, handler: impl Handler + 'static) -> ConfigBuilder<Arc<dyn Handler>, ...> {
    ConfigBuilder {
        handler: Arc::new(handler),
        ...
    }
}
```

**Implementation approach:**

```rust
fn generate_builder_methods(builder_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
    let methods: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let method_name = Ident::new(&format!("with_{}", field_name), field_name.span());

            // Analyze field type for wrapping
            let wrapper_type = analyze_field_type(field_type);

            match wrapper_type {
                FieldWrapperType::Arc(trait_ty) => {
                    // Generate method with impl Trait parameter
                    // Use Arc::new() in field assignment
                    quote! {
                        pub fn #method_name(
                            self,
                            #field_name: impl #trait_ty + 'static
                        ) -> #builder_name<#(#return_type_params),*> {
                            #builder_name {
                                #field_name: std::sync::Arc::new(#field_name),
                                #(#other_field_assignments),*
                            }
                        }
                    }
                }

                FieldWrapperType::Box(trait_ty) => {
                    // Generate method with impl Trait parameter
                    // Use Box::new() in field assignment
                    quote! {
                        pub fn #method_name(
                            self,
                            #field_name: impl #trait_ty + 'static
                        ) -> #builder_name<#(#return_type_params),*> {
                            #builder_name {
                                #field_name: Box::new(#field_name),
                                #(#other_field_assignments),*
                            }
                        }
                    }
                }

                FieldWrapperType::None => {
                    // Original behavior - no wrapping
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

    // ... rest of implementation
}
```

### 3. Type Parsing Details

**Challenge:** Parse `Arc<dyn Trait>` from `syn::Type`

The type structure looks like:
```
Type::Path {
    path: Path {
        segments: [
            PathSegment {
                ident: "Arc",
                arguments: PathArguments::AngleBracketed {
                    args: [
                        GenericArgument::Type(
                            Type::TraitObject {
                                bounds: [TypeParamBound::Trait(...)]
                            }
                        )
                    ]
                }
            }
        ]
    }
}
```

**Implementation:**

```rust
fn is_arc_dyn_trait(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        // Check if last segment is "Arc"
        let last_segment = type_path.path.segments.last()?;

        if last_segment.ident != "Arc" {
            return None;
        }

        // Get generic arguments
        if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
            if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                // Check if inner type is TraitObject (has dyn)
                if matches!(inner_ty, Type::TraitObject(_)) {
                    return Some(inner_ty);
                }
            }
        }
    }

    None
}

fn extract_trait_from_dyn(ty: &Type) -> Option<&Type> {
    if let Type::TraitObject(trait_obj) = ty {
        // Get first bound (main trait)
        if let Some(TypeParamBound::Trait(trait_bound)) = trait_obj.bounds.first() {
            // Convert trait bound back to Type for quote!
            // This is tricky - we need the trait path
            // We can reconstruct it or use the bound directly
            return Some(ty); // For now, return the whole trait object
        }
    }

    None
}
```

**Alternative approach using quote and string manipulation:**

We might need to extract just the trait name from `dyn Trait`. This could involve:
1. Converting the trait object to tokens
2. Removing the `dyn` keyword
3. Reconstructing as a trait bound for `impl Trait`

### 4. Handle Edge Cases

**Multiple trait bounds:**
```rust
Arc<dyn Handler + Send + Sync>
```

Should generate:
```rust
pub fn with_handler(self, handler: impl Handler + Send + Sync + 'static) -> ...
```

**Implementation:**
```rust
fn extract_all_bounds(trait_obj: &TypeTraitObject) -> Vec<&TypeParamBound> {
    trait_obj.bounds.iter().collect()
}

// In method generation:
quote! {
    impl #(#bounds)+* + 'static
}
```

**Qualified paths:**
```rust
Arc<dyn some_crate::Handler>
```

Should work as-is since we preserve the full path.

**Lifetime parameters:**
```rust
Arc<dyn Handler + 'a>  // Has explicit lifetime
```

Should preserve the lifetime in `impl` parameter:
```rust
impl Handler + 'a
```

### 5. Testing Strategy

Create comprehensive tests in `tests/builder.rs`:

```rust
// Test 1: Basic Arc<dyn Trait>
trait Logger {
    fn log(&self, msg: &str);
}

struct ConsoleLogger;
impl Logger for ConsoleLogger {
    fn log(&self, msg: &str) {
        println!("{}", msg);
    }
}

#[derive(Builder)]
struct App {
    logger: Arc<dyn Logger>,
}

#[test]
fn test_arc_dyn_trait_auto_wrap() {
    let app = AppBuilder::new()
        .with_logger(ConsoleLogger)  // No manual Arc::new!
        .build();
}

// Test 2: Box<dyn Trait>
#[derive(Builder)]
struct Config {
    handler: Box<dyn Logger>,
}

#[test]
fn test_box_dyn_trait_auto_wrap() {
    let config = ConfigBuilder::new()
        .with_handler(ConsoleLogger)  // No manual Box::new!
        .build();
}

// Test 3: Multiple bounds
#[derive(Builder)]
struct Worker {
    processor: Arc<dyn Processor + Send + Sync>,
}

#[test]
fn test_multi_bound_auto_wrap() {
    let worker = WorkerBuilder::new()
        .with_processor(MyProcessor)
        .build();
}

// Test 4: Mixed - some wrapped, some not
#[derive(Builder)]
struct Service {
    name: String,
    logger: Arc<dyn Logger>,
    port: u16,
}

#[test]
fn test_mixed_fields() {
    let service = ServiceBuilder::new()
        .with_name("api".to_string())
        .with_logger(ConsoleLogger)  // Auto-wrapped
        .with_port(8080)
        .build();
}
```

### 6. Update Documentation

**README.md updates:**

Add new section:

```markdown
### Smart Wrapping for Trait Objects

The builder automatically wraps trait objects in `Arc` or `Box`:

\```rust
trait Logger {
    fn log(&self, msg: &str);
}

#[derive(Builder)]
struct App {
    logger: Arc<dyn Logger>,
}

// No need to manually wrap!
let app = AppBuilder::new()
    .with_logger(ConsoleLogger::new())  // Automatically wrapped in Arc
    .build();
\```

This works for:
- `Arc<dyn Trait>`
- `Box<dyn Trait>`
- Traits with multiple bounds: `Arc<dyn Trait + Send + Sync>`
```

## Implementation Complexity

### Low Complexity
- Detecting `Arc` vs `Box` wrapper
- Basic `dyn Trait` detection
- Generating `Arc::new()` / `Box::new()` calls

### Medium Complexity
- Extracting trait from `Type::TraitObject`
- Handling multiple trait bounds correctly
- Preserving trait paths (e.g., `some_crate::Trait`)

### High Complexity
- Converting `TypeTraitObject` to trait bound for `impl Trait`
- Handling lifetime parameters correctly
- Edge cases with qualified paths

## Potential Issues

### Issue 1: Trait Object to Impl Conversion

**Problem:** `Type::TraitObject` (e.g., `dyn Trait`) is different from what we need for `impl Trait`.

**Solution:**
- Parse the bounds from `TypeTraitObject`
- Reconstruct them for the `impl` parameter
- Use `quote!` to generate: `impl #(#bounds)+* + 'static`

### Issue 2: Adding 'static Lifetime

**Problem:** `Arc::new()` and `Box::new()` require `'static` for trait objects.

**Solution:**
- Always add `+ 'static` to impl bounds
- Unless the original trait object has a different explicit lifetime
- Check trait bounds for existing lifetime, preserve if found

### Issue 3: Type Imports

**Problem:** Need `std::sync::Arc` in generated code.

**Solution:**
- Use fully qualified paths: `std::sync::Arc::new()`
- Or assume `Arc` is in scope (user responsibility)
- Document that `use std::sync::Arc;` might be needed

## Alternative Approaches

### Option 1: Always Require Manual Wrapping
- Simpler implementation
- More explicit, less magic
- **Rejected:** Less ergonomic

### Option 2: Provide Both Methods
- Generate both `with_handler()` (auto-wrap) and `with_handler_raw()` (manual)
- More flexible but more verbose
- **Considered:** Could be added later if needed

### Option 3: Conditional Feature Flag
- Make auto-wrapping opt-in via feature flag
- **Considered:** Adds complexity, probably not needed

## Recommendation

Implement the smart wrapping feature as described, focusing on:

1. **Phase 1** (Essential):
   - Detect `Arc<dyn Trait>` and `Box<dyn Trait>`
   - Generate `impl Trait + 'static` parameters
   - Auto-wrap with `Arc::new()` / `Box::new()`

2. **Phase 2** (Nice-to-have):
   - Support multiple trait bounds
   - Handle qualified paths
   - Preserve explicit lifetimes

3. **Phase 3** (If needed):
   - Support more complex patterns
   - Additional wrapper types if requested

Start with Phase 1 for MVP, then iterate based on testing.
