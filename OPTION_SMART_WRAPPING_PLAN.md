# Option<T> Smart Wrapping Implementation Plan

## Goal
Add automatic handling for `Option<T>` fields to improve ergonomics:
- Fields of type `Option<T>` are initialized to `None` by default
- Builder methods accept `impl Into<Option<T>>`
- Users can pass either `T` (automatically wrapped in `Some`) or `Option<T>` (including `None`)

## Design Decisions

### 1. Option Fields as Implicitly Optional
**Decision**: All `Option<T>` fields are treated as having an implicit `#[default(None)]`

**Rationale**:
- Most ergonomic - matches developer expectations
- `Option<T>` semantically means "optional value"
- If a field must be required, use `T` directly instead of `Option<T>`

**Behavior**:
- `Option<T>` fields don't participate in typestate progression
- They're always optional (can build without setting them)
- Initialized to `None` in `new()`
- Can be set at any time using mutable consuming pattern

### 2. Builder Method Signature: `impl Into<Option<T>>`

**Decision**: Builder methods for `Option<T>` fields accept `impl Into<Option<T>>`

**Rationale**:
- Rust implements `From<T> for Option<T>`, which provides automatic `Into<Option<T>>`
- `Option<T>` also implements `Into<Option<T>>` (identity conversion)
- Maximum flexibility: accept both `T` and `Option<T>`

**Examples**:
```rust
// All of these work:
builder.with_description("text".to_string())     // T -> Some(T)
builder.with_description(Some("text".to_string())) // Option<T> -> Option<T>
builder.with_description(None)                   // Option<T> -> None
```

**Benefits**:
- Ergonomic: Can pass bare values for common case
- Explicit control: Can pass `None` explicitly if needed
- Type-safe: Compiler ensures correct types

### 3. Interaction with Existing Attributes

#### With `#[default(value)]`
```rust
#[default(Some(42))]
count: Option<i32>,
```
**Behavior**: Explicit default takes precedence
- Initialize with `Some(42)` instead of `None`
- Builder method still accepts `impl Into<Option<i32>>`
- This allows non-None defaults

#### With `#[incomplete]`
```rust
#[incomplete]
optional_field: Option<String>,
```
**Behavior**: Redundant but allowed
- `#[incomplete]` has no effect since Option fields are already optional
- Treated same as unmarked `Option<T>` field
- No error or warning (silent no-op)

#### With both `#[default]` and `#[incomplete]`
```rust
#[default(Some(value))]
#[incomplete]
field: Option<T>,
```
**Behavior**: Error (already mutually exclusive)
- Existing validation catches this
- Clear error message shown to user

### 3. Nested Option Types
```rust
nested: Option<Option<T>>,
```
**Behavior**: Only unwrap outer Option
- Detect only the outermost `Option<T>`
- Builder method accepts `Option<T>`, wraps in outer `Some`
- Avoids confusion with double-unwrapping

## Implementation Steps

### Phase 1: Type Detection
**File**: `src/lib.rs`

Add function to detect `Option<T>` and extract inner type:
```rust
/// Check if a type is Option<T> and extract the inner type
fn is_option_type(ty: &Type) -> Option<Type> {
    if let Type::Path(type_path) = ty {
        let last_segment = type_path.path.segments.last()?;

        if last_segment.ident != "Option" {
            return None;
        }

        if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
            if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                return Some(inner_ty.clone());
            }
        }
    }

    None
}
```

### Phase 2: Update WrapperType Enum
**File**: `src/lib.rs`

Add `Option` variant to `WrapperType`:
```rust
enum WrapperType {
    None,
    Arc(Vec<TypeParamBound>),
    Box(Vec<TypeParamBound>),
    Option(Box<Type>),  // NEW: Store inner type
}
```

Update `analyze_field_type()`:
```rust
fn analyze_field_type(ty: &Type) -> WrapperType {
    if let Some(bounds) = is_arc_dyn_trait(ty) {
        return WrapperType::Arc(bounds);
    }

    if let Some(bounds) = is_box_dyn_trait(ty) {
        return WrapperType::Box(bounds);
    }

    if let Some(inner_ty) = is_option_type(ty) {
        return WrapperType::Option(Box::new(inner_ty));
    }

    WrapperType::None
}
```

### Phase 3: Update Field Categorization
**File**: `src/lib.rs`

Modify `categorize_field()` to treat `Option<T>` as default:
```rust
fn categorize_field(field: &Field) -> syn::Result<CategorizedField> {
    let name = field.ident.as_ref().unwrap().clone();
    let ty = field.ty.clone();
    let wrapper = analyze_field_type(&ty);
    let has_incomplete = has_incomplete_attr(field);
    let default_value = extract_default_value(field);

    // Mutual exclusivity check (existing)
    if default_value.is_some() && has_incomplete {
        return Err(syn::Error::new_spanned(
            field,
            "field cannot have both #[default] and #[incomplete] attributes"
        ));
    }

    // If explicit default is provided, use it
    if let Some(default_value) = default_value {
        return Ok(CategorizedField {
            name,
            category: FieldCategory::Default { value: default_value, ty },
            wrapper,
            has_incomplete: false,
        });
    }

    // NEW: If field is Option<T> without explicit default, treat as Default with None
    if matches!(wrapper, WrapperType::Option(_)) {
        return Ok(CategorizedField {
            name,
            category: FieldCategory::Default {
                value: syn::parse_quote!(None),
                ty
            },
            wrapper,
            has_incomplete: false,
        });
    }

    // Regular field
    Ok(CategorizedField {
        name,
        category: FieldCategory::Regular { ty },
        wrapper,
        has_incomplete,
    })
}
```

### Phase 4: Update Builder Method Generation
**File**: `src/lib.rs`

Add `Option` handling to both regular and default method generation:

```rust
// In generate_builder_methods(), add Option case:
WrapperType::Option(inner_ty) => {
    quote! {
        pub fn #method_name(mut self, #field_name: impl Into<Option<#inner_ty>>) -> Self {
            self.#field_name = #field_name.into();
            self
        }
    }
}
```

**Key aspects**:
- Parameter type: `impl Into<Option<T>>`
- Assignment: `self.field = value.into()` - relies on Into trait
- This automatically handles both `T` and `Option<T>` inputs

### Phase 5: Complex Combinations
Handle combinations of Option with Arc/Box:
```rust
field: Option<Arc<dyn Trait>>
```

**Approach**: Check for Option first, then check inner type for Arc/Box
- Builder accepts `impl Trait`
- Wraps in `Arc::new()` then converts via `Into<Option<_>>`
- Can also accept `Option<Arc<dyn Trait>>` explicitly

**Update `analyze_field_type()`**:
```rust
fn analyze_field_type(ty: &Type) -> WrapperType {
    // Check for Option<Arc<dyn Trait>> or Option<Box<dyn Trait>>
    if let Some(inner_ty) = is_option_type(ty) {
        if let Some(bounds) = is_arc_dyn_trait(&inner_ty) {
            return WrapperType::OptionArc(bounds);  // NEW variant
        }
        if let Some(bounds) = is_box_dyn_trait(&inner_ty) {
            return WrapperType::OptionBox(bounds);  // NEW variant
        }
        return WrapperType::Option(Box::new(inner_ty));
    }

    // ... rest of checks
}
```

Add new enum variants:
```rust
enum WrapperType {
    None,
    Arc(Vec<TypeParamBound>),
    Box(Vec<TypeParamBound>),
    Option(Box<Type>),
    OptionArc(Vec<TypeParamBound>),  // NEW
    OptionBox(Vec<TypeParamBound>),  // NEW
}
```

**Code generation for OptionArc/OptionBox**:
```rust
// For Option<Arc<dyn Trait>>
WrapperType::OptionArc(bounds) => {
    let trait_bounds = quote! { #(#bounds)+* + 'static };
    quote! {
        pub fn #method_name(
            mut self,
            #field_name: impl Into<Option<impl #trait_bounds>>
        ) -> Self {
            self.#field_name = #field_name.into().map(|v| std::sync::Arc::new(v));
            self
        }
    }
}

// For Option<Box<dyn Trait>>
WrapperType::OptionBox(bounds) => {
    let trait_bounds = quote! { #(#bounds)+* + 'static };
    quote! {
        pub fn #method_name(
            mut self,
            #field_name: impl Into<Option<impl #trait_bounds>>
        ) -> Self {
            self.#field_name = #field_name.into().map(|v| Box::new(v));
            self
        }
    }
}
```

**Usage examples**:
```rust
// All of these work:
builder.with_logger(ConsoleLogger)                      // T -> Some(Arc::new(T))
builder.with_logger(Some(ConsoleLogger))                // Option<T> -> Option<Arc<T>>
builder.with_logger(None)                               // None -> None
builder.with_logger(Some(Arc::new(ConsoleLogger)))      // Option<Arc<T>> -> Option<Arc<T>>
```

## Testing Strategy

### Test Cases to Add

1. **Basic Option field**
```rust
#[derive(Builder)]
struct Config {
    name: String,
    description: Option<String>,
}

// All of these work:
ConfigBuilder::new()
    .with_name("test".to_string())
    .build(); // description is None

ConfigBuilder::new()
    .with_name("test".to_string())
    .with_description("desc".to_string())  // Takes String -> Some("desc")
    .build();

ConfigBuilder::new()
    .with_name("test".to_string())
    .with_description(Some("desc".to_string()))  // Takes Option<String>
    .build();

ConfigBuilder::new()
    .with_name("test".to_string())
    .with_description(None)  // Explicitly set to None
    .build();
```

2. **Multiple Option fields**
```rust
#[derive(Builder)]
struct Config {
    required: String,
    opt1: Option<i32>,
    opt2: Option<bool>,
}
```

3. **Option with explicit default**
```rust
#[derive(Builder)]
struct Config {
    name: String,
    #[default(Some(42))]
    count: Option<i32>,
}
```

4. **Option with Arc/Box trait object**
```rust
#[derive(Builder)]
struct Config {
    name: String,
    logger: Option<Arc<dyn Logger>>,
}

// Should accept: impl Logger
// Should wrap as: Some(Arc::new(logger))
```

5. **Option with incomplete (redundant)**
```rust
#[derive(Builder)]
struct Config {
    name: String,
    #[incomplete]
    optional: Option<String>,
}
// Should work same as without #[incomplete]
```

6. **Complex types in Option**
```rust
#[derive(Builder)]
struct Config {
    name: String,
    tags: Option<Vec<String>>,
    metadata: Option<HashMap<String, String>>,
}
```

7. **Nested Option (edge case)**
```rust
#[derive(Builder)]
struct Config {
    name: String,
    nested: Option<Option<String>>,
}
// Builder should accept Option<String>
```

## Documentation Updates

### README.md
Add section after "Smart Wrapping for Trait Objects":

```markdown
### Smart Wrapping for Option Types

Fields of type `Option<T>` are automatically treated as optional with smart wrapping:

\`\`\`rust
#[derive(Builder)]
struct ServerConfig {
    host: String,
    port: u16,
    description: Option<String>,
    max_connections: Option<usize>,
}

fn main() {
    // Build without optional fields - they default to None
    let config1 = ServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(8080)
        .build();

    assert_eq!(config1.description, None);
    assert_eq!(config1.max_connections, None);

    // Set optional fields - pass T directly (most ergonomic)
    let config2 = ServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(8080)
        .with_description("My Server".to_string())  // T -> Some(T)
        .with_max_connections(100)  // T -> Some(T)
        .build();

    assert_eq!(config2.description, Some("My Server".to_string()));
    assert_eq!(config2.max_connections, Some(100));

    // Can also pass Option<T> explicitly
    let config3 = ServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(8080)
        .with_description(Some("Explicit".to_string()))  // Option<T> -> Option<T>
        .with_max_connections(None)  // Can explicitly set to None
        .build();

    assert_eq!(config3.description, Some("Explicit".to_string()));
    assert_eq!(config3.max_connections, None);
}
\`\`\`

**Key behaviors:**
- `Option<T>` fields are initialized to `None` by default
- Builder methods accept `impl Into<Option<T>>`
- Can pass `T` (wrapped in `Some` automatically) or `Option<T>` (including `None`)
- Maximum flexibility: ergonomic for common case, explicit control when needed
- Can be combined with trait object wrapping: `Option<Arc<dyn Trait>>`
- Can override default with `#[default(Some(value))]`
```

## Edge Cases and Considerations

1. **Type Aliases**
```rust
type MaybeString = Option<String>;
field: MaybeString,
```
**Behavior**: Won't be detected (requires type resolution)
**Decision**: Document as limitation

2. **Fully Qualified Paths**
```rust
field: std::option::Option<String>,
```
**Behavior**: Won't be detected (checks for "Option" ident only)
**Decision**: Document to use `Option<T>` directly

3. **Generic Option Types**
```rust
#[derive(Builder)]
struct Config<T> {
    value: Option<T>,
}
```
**Behavior**: Should work - inner type is generic parameter
**Decision**: Add test case

4. **Result vs Option**
```rust
field: Result<T, E>,
```
**Behavior**: Not handled (different semantics)
**Decision**: Only handle `Option<T>`, not `Result<T, E>`

## Summary

This implementation provides ergonomic handling for optional fields while maintaining type safety and consistency with existing features. The key insights are:

1. **Implicit Default**: Treating `Option<T>` as having an implicit `#[default(None)]` aligns with developer expectations
2. **Into Trait Pattern**: Using `impl Into<Option<T>>` provides maximum flexibility - both `T` and `Option<T>` work seamlessly

**Benefits**:
- **Most Ergonomic**: Pass bare values for common case (`"text"` instead of `Some("text")`)
- **Explicit Control**: Can still pass `None` explicitly when needed
- **Type Safe**: Compiler ensures correct types via Into trait
- **Consistent**: Aligns with Arc/Box smart wrapping pattern
- **Flexible**: Works with all combinations (Option, Arc, Box, etc.)

**Minimal Breaking Changes**:
- Existing code without `Option<T>` fields unaffected
- Option fields that weren't working before now work better
- No API changes to existing functionality
