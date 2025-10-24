# Rust Config Proc-Macro Implementation Plan

## Overview
Implement a `#[derive(Config)]` procedural macro that generates type-safe configuration boilerplate using the **typestate pattern**. This ensures compile-time verification that all required configuration fields are properly initialized.

## Requirements Analysis

### Input Structure (`config_macro.rs`)
```rust
#[derive(Config)]
struct Config {
    url: url::Url,
    name: String,
    #[incomplete]
    secret: zeroize::Zeroizing<String>,
}
```

### Generated Output (`config_generated.rs`)
The macro must generate:
1. A generic struct with type parameters for each field
2. Type aliases for configurations:
   - `{StructName}Complete`: Always generated with all original types
   - `{StructName}Incomplete`: Only generated if `#[incomplete]` markers exist
   - Multiple `#[incomplete]` fields all become `()` in the incomplete alias
3. A `new()` constructor returning all fields as `()`
4. Builder methods (`with_*`) for each field

**Key Rules:**
- Type alias names adapt to struct name (e.g., `Foo` → `FooComplete`, `FooIncomplete`)
- `{StructName}Incomplete` is NOT generated if there are no `#[incomplete]` markers
- All fields marked `#[incomplete]` become `()` in the same incomplete type alias

## Implementation Steps

### 1. Project Setup and Configuration
**Files to modify:** `Cargo.toml`

- [ ] Set correct Rust edition (`2021` instead of `2024`)
- [ ] Add `proc-macro = true` to package section
- [ ] Add required dependencies:
  - `syn` with features `["full", "derive"]`
  - `quote`
  - `proc-macro2`

### 2. Create Proc-Macro Entry Point
**Files to create/modify:** `src/lib.rs`

- [ ] Remove existing module declarations
- [ ] Create `#[proc_macro_derive(Config, attributes(incomplete))]` entry point
- [ ] Parse input using `syn::parse_macro_input!`
- [ ] Call code generation function
- [ ] Return generated `TokenStream`

### 3. Parse Input Structure
**Module to create:** `src/parser.rs` or inline in `lib.rs`

- [ ] Extract struct name from `DeriveInput`
- [ ] Extract struct fields from `syn::Fields::Named`
- [ ] For each field, capture:
  - Field name (identifier)
  - Field type (full type path)
  - Whether it has `#[incomplete]` attribute
- [ ] Validate input (must be a struct with named fields)
- [ ] Return structured representation of parsed data

### 4. Generate Generic Struct Definition
**Function:** `generate_generic_struct()`

- [ ] Create generic type parameters (PascalCase from field names)
  - Example: `url` → `Url`, `name` → `Name`, `secret` → `Secret`
- [ ] Transform original struct into generic version:
  ```rust
  struct Config<Url, Name, Secret> {
      url: Url,
      name: Name,
      secret: Secret,
  }
  ```

### 5. Generate Type Aliases
**Function:** `generate_type_aliases()`

- [ ] Generate `{StructName}Complete` type alias (always):
  - Use all original field types
  - Name adapts to struct name (e.g., `Config` → `ConfigComplete`, `Foo` → `FooComplete`)
  ```rust
  pub type ConfigComplete = Config<url::Url, String, zeroize::Zeroizing<String>>;
  ```

- [ ] Generate `{StructName}Incomplete` type alias (only if `#[incomplete]` markers exist):
  - Fields with `#[incomplete]` attribute → `()`
  - All other fields → original types
  - If multiple fields have `#[incomplete]`, they all become `()` in the same type alias
  - Name adapts to struct name (e.g., `Config` → `ConfigIncomplete`, `Foo` → `FooIncomplete`)
  - **Important:** Skip this entirely if no `#[incomplete]` markers are present
  ```rust
  pub type ConfigIncomplete = Config<url::Url, String, ()>;
  ```

### 6. Generate Constructor
**Function:** `generate_new_constructor()`

- [ ] Create `impl Config<(), (), ()>` block
- [ ] Generate `new()` method returning `Self` with all fields as `()`
  ```rust
  impl Config<(), (), ()> {
      pub fn new() -> Self {
          Self {
              url: (),
              name: (),
              secret: (),
          }
      }
  }
  ```

### 7. Generate Builder Methods
**Function:** `generate_builder_methods()`

- [ ] Create `impl<Url, Name, Secret> Config<Url, Name, Secret>` block
- [ ] For each field, generate a `with_{field_name}` method:
  - Takes `self` by value (consuming)
  - Accepts parameter of the field's original type
  - Returns `Config` with that field's generic replaced by concrete type
  - Preserves all other fields

  Example for `url` field:
  ```rust
  pub fn with_url(self, url: url::Url) -> Config<url::Url, Name, Secret> {
      Config {
          url,
          name: self.name,
          secret: self.secret,
      }
  }
  ```

### 8. Testing Strategy

- [ ] Create test module in `lib.rs` or separate `tests/` directory
- [ ] Test basic struct with all required fields (no `#[incomplete]` markers)
  - Verify `{StructName}Complete` is generated
  - Verify `{StructName}Incomplete` is NOT generated
- [ ] Test struct with single `#[incomplete]` attribute
  - Verify both type aliases are generated correctly
- [ ] Test struct with multiple `#[incomplete]` attributes
  - Verify all marked fields become `()` in the incomplete alias
- [ ] Test dynamic naming (struct named something other than `Config`)
  - Verify type aliases match struct name
- [ ] Test that builder pattern compiles correctly
- [ ] Test that incomplete configurations cannot be used where complete ones are needed
- [ ] Verify generated code matches expected output

### 9. Error Handling

- [ ] Handle invalid input (not a struct)
- [ ] Handle tuple structs or unit structs (should error)
- [ ] Handle duplicate `#[incomplete]` attributes gracefully
- [ ] Provide helpful error messages using `syn::Error`

## Technical Implementation Details

### Helper Functions Needed

1. **`extract_incomplete_attribute(field: &Field) -> bool`**
   - Check if field has `#[incomplete]` attribute

2. **`has_incomplete_fields(fields: &[Field]) -> bool`**
   - Check if ANY field has `#[incomplete]` attribute
   - Used to determine if incomplete type alias should be generated

3. **`field_name_to_type_param(name: &Ident) -> Ident`**
   - Convert field name to PascalCase type parameter
   - Example: `url` → `Url`

4. **`generate_generic_params(fields: &[Field]) -> Vec<Ident>`**
   - Create list of generic type parameters

5. **`generate_field_assignments(fields: &[Field], exclude: Option<&Ident>) -> TokenStream`**
   - Generate field assignments for struct construction
   - Used in builder methods to preserve fields

6. **`create_type_alias_name(struct_name: &Ident, suffix: &str) -> Ident`**
   - Create type alias name from struct name
   - Example: `Config` + `"Complete"` → `ConfigComplete`

### Code Organization

```
src/
├── lib.rs              # Proc-macro entry point
├── parser.rs           # Input parsing logic (optional)
├── generator.rs        # Code generation functions (optional)
└── utils.rs            # Helper functions (optional)
```

For simplicity, can implement everything in `lib.rs` initially.

## Expected Behavior

### Usage Example
```rust
let config = Config::new()
    .with_url("https://example.com".parse().unwrap())
    .with_name("MyApp".to_string())
    .with_secret(zeroize::Zeroizing::new("secret".to_string()));

// Type is: Config<url::Url, String, zeroize::Zeroizing<String>>
// Which matches ConfigComplete
```

### Type Safety
```rust
let incomplete = Config::new()
    .with_url("https://example.com".parse().unwrap())
    .with_name("MyApp".to_string());

// Type is: Config<url::Url, String, ()>
// Which matches ConfigIncomplete (only generated if #[incomplete] markers exist)
// Compiler prevents using this where ConfigComplete is required
```

### Dynamic Naming
```rust
#[derive(Config)]
struct DatabaseConfig {
    host: String,
    #[incomplete]
    password: String,
}

// Generates:
// - DatabaseConfigComplete = DatabaseConfig<String, String>
// - DatabaseConfigIncomplete = DatabaseConfig<String, ()>
```

## Potential Challenges

1. **Type Parameter Naming**: Ensure generated type parameters don't conflict
2. **Attribute Parsing**: Correctly identify `#[incomplete]` attributes across multiple fields
3. **Conditional Type Alias Generation**: Only generate `{StructName}Incomplete` when needed
4. **Dynamic Naming**: Properly format struct name into type alias names
5. **Quote Hygiene**: Proper use of `quote!` macro to generate valid Rust code
6. **Field Ordering**: Maintain consistent field order in all generated code
7. **Type Path Handling**: Preserve full type paths like `url::Url` and `zeroize::Zeroizing<String>`

## Success Criteria

- [ ] Macro compiles without errors
- [ ] Generated code matches expected output in `config_generated.rs`
- [ ] Code in `config_macro.rs` successfully derives the Config trait
- [ ] All builder methods work correctly
- [ ] Type aliases are correctly generated
- [ ] Incomplete fields are properly handled

## Next Steps

After approval of this plan:
1. Update `Cargo.toml` with correct configuration
2. Implement the proc-macro in `src/lib.rs`
3. Test the implementation
4. Commit and push changes to the development branch
