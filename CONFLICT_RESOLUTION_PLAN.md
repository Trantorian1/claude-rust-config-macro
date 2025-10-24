# Conflict Resolution Plan: Struct Redefinition Issue

## Problem Statement

The current implementation uses a **derive macro** (`#[derive(Config)]`) which can only *add* code, not replace existing definitions. This causes a conflict:

```rust
// User writes this:
#[derive(Config)]
struct Config {
    url: url::Url,
    name: String,
    #[incomplete]
    secret: zeroize::Zeroizing<String>,
}

// Macro generates this (conflict!):
struct Config<Url, Name, Secret> {  // ❌ Same name, different signature
    url: Url,
    name: Name,
    secret: Secret,
}
```

**Result**: Two structs named `Config` with conflicting definitions exist in the same scope.

## Root Cause

**Derive macros** (`#[derive(...)]`) can only:
- ✅ Add trait implementations
- ✅ Generate new types/functions
- ❌ **Cannot replace or modify the item they're applied to**

The original struct remains intact, and our generated generic struct conflicts with it.

## Solution Options

### Option 1: Use Attribute Macro (RECOMMENDED)
**Change from `#[derive(Config)]` to `#[config]`**

**Pros**:
- Attribute macros can completely replace the item they're applied to
- Clean solution - only one struct definition exists
- Matches the expected output in `config_generated.rs` exactly
- Most intuitive for users

**Cons**:
- Requires changing the macro type from derive to attribute
- Syntax changes from `#[derive(Config)]` to `#[config]`

**Implementation**:
```rust
// User writes:
#[config]
struct Config {
    url: url::Url,
    name: String,
    #[incomplete]
    secret: zeroize::Zeroizing<String>,
}

// Macro replaces entire definition with:
struct Config<Url, Name, Secret> {
    url: Url,
    name: Name,
    secret: Secret,
}
// + type aliases + impls...
```

### Option 2: Generate Different Name + Type Alias
**Generate `ConfigBuilder<...>` instead of `Config<...>`**

**Pros**:
- Can keep using derive macro
- Original struct untouched (though unused)

**Cons**:
- Two struct definitions still exist (original unused)
- Generated names less intuitive (`ConfigBuilder` instead of `Config`)
- Doesn't match the expected output in `config_generated.rs`
- More complex type aliases needed

**Implementation**:
```rust
// Original struct remains:
struct Config { ... }  // Unused

// Macro generates:
struct ConfigBuilder<Url, Name, Secret> { ... }
pub type ConfigComplete = ConfigBuilder<url::Url, String, zeroize::Zeroizing<String>>;
pub type ConfigIncomplete = ConfigBuilder<url::Url, String, ()>;
```

### Option 3: Use Module Wrapping
**Generate everything in a new module**

**Pros**:
- Can use derive macro
- Clear separation

**Cons**:
- Awkward ergonomics (need to import from submodule)
- Doesn't match expected output
- Still have unused original struct

**Implementation**:
```rust
struct Config { ... }  // Original, unused

mod config_builder {
    pub struct Config<Url, Name, Secret> { ... }
    // ...
}
```

### Option 4: Suppress Original Struct (Not Possible)
**Try to "hide" the original struct**

**Verdict**: ❌ Not possible - proc-macros cannot modify or remove existing items.

## Recommended Solution: Builder Pattern with `build()` Method

### Overview

Keep `#[derive(Config)]` but generate a separate `{StructName}Builder` struct that works alongside the original struct.

**Key Design**:
1. Original struct remains untouched and usable
2. Generate `{StructName}Builder<...>` with generic type parameters
3. Replace `{StructName}Complete` type alias with a `build()` method that returns the original struct
4. Keep `{StructName}Incomplete` as a type alias pointing to the builder with `()` for incomplete fields
5. The `build()` method is only available when all fields are concrete (trait-bound constrained)

### Generated Code Structure

```rust
// User writes:
#[derive(Config)]
struct Config {
    url: url::Url,
    name: String,
    #[incomplete]
    secret: zeroize::Zeroizing<String>,
}

// Original struct remains as-is (user can use it directly)

// Macro generates:
struct ConfigBuilder<Url, Name, Secret> {
    url: Url,
    name: Name,
    secret: Secret,
}

// Type alias for incomplete state
pub type ConfigIncomplete = ConfigBuilder<url::Url, String, ()>;

// Constructor returns builder with all () fields
impl ConfigBuilder<(), (), ()> {
    pub fn new() -> Self {
        Self {
            url: (),
            name: (),
            secret: (),
        }
    }
}

// Builder methods (same as before)
impl<Url, Name, Secret> ConfigBuilder<Url, Name, Secret> {
    pub fn with_url(self, url: url::Url) -> ConfigBuilder<url::Url, Name, Secret> {
        ConfigBuilder {
            url,
            name: self.name,
            secret: self.secret,
        }
    }

    pub fn with_name(self, name: String) -> ConfigBuilder<Url, String, Secret> {
        ConfigBuilder {
            url: self.url,
            name,
            secret: self.secret,
        }
    }

    pub fn with_secret(
        self,
        secret: zeroize::Zeroizing<String>,
    ) -> ConfigBuilder<Url, Name, zeroize::Zeroizing<String>> {
        ConfigBuilder {
            url: self.url,
            name: self.name,
            secret,
        }
    }
}

// build() method - only available when all fields are concrete
impl ConfigBuilder<url::Url, String, zeroize::Zeroizing<String>> {
    pub fn build(self) -> Config {
        Config {
            url: self.url,
            name: self.name,
            secret: self.secret,
        }
    }
}
```

### Why This Approach?

**Pros**:
- ✅ Keeps `#[derive(Config)]` syntax - no breaking changes
- ✅ No struct name conflicts - builder has distinct name
- ✅ Original struct remains usable for direct construction
- ✅ Clear separation: `Config` = final type, `ConfigBuilder` = construction mechanism
- ✅ Type safety preserved through typestate pattern
- ✅ `build()` method only available when complete (type-safe)
- ✅ Standard builder pattern familiar to Rust developers

**Cons**:
- Users must call `.build()` at the end
- Slightly different API than original plan (but more idiomatic)

### Usage Example

```rust
// Build through typestate pattern
let config = ConfigBuilder::new()
    .with_url("https://example.com".parse().unwrap())
    .with_name("MyApp".to_string())
    .with_secret(zeroize::Zeroizing::new("secret".to_string()))
    .build();  // Returns Config

// Type checking works
let incomplete: ConfigIncomplete = ConfigBuilder::new()
    .with_url("https://example.com".parse().unwrap())
    .with_name("MyApp".to_string());
// incomplete.build() would not compile - build() method not available!

// Direct construction still works
let config = Config {
    url: "https://example.com".parse().unwrap(),
    name: "MyApp".to_string(),
    secret: zeroize::Zeroizing::new("secret".to_string()),
};
```

### Implementation Changes Required

#### 1. Update Struct Generation
Change from generating `{StructName}<...>` to `{StructName}Builder<...>`:

```rust
fn generate_generic_struct(struct_name: &Ident, fields: &[Field]) -> proc_macro2::TokenStream {
    let builder_name = format!("{}Builder", struct_name);
    let builder_ident = Ident::new(&builder_name, struct_name.span());

    // ... rest of generation using builder_ident
}
```

#### 2. Remove `{StructName}Complete` Type Alias
Replace with specialized `build()` method implementation.

#### 3. Update `{StructName}Incomplete` Type Alias
Point to `{StructName}Builder` instead:

```rust
pub type ConfigIncomplete = ConfigBuilder<url::Url, String, ()>;
```

#### 4. Generate `build()` Method
Create specialized impl block for fully concrete types:

```rust
fn generate_build_method(
    struct_name: &Ident,
    builder_name: &Ident,
    fields: &[Field]
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
```

#### 5. Update Constructor
Constructor returns `{StructName}Builder`, not `{StructName}`:

```rust
impl ConfigBuilder<(), (), ()> {
    pub fn new() -> Self { ... }
}
```

### Migration Path

1. Update `src/lib.rs`:
   - Change struct generation to use `{StructName}Builder`
   - Remove `{StructName}Complete` type alias generation
   - Update `{StructName}Incomplete` to point to builder
   - Add `build()` method generation
   - Update all references from struct name to builder name in impls

2. Update `config_generated.rs`:
   - Update to show expected output with builder pattern

3. Update tests:
   - Change type annotations from `ConfigComplete` to use `.build()`
   - Update incomplete type assertions to use `ConfigIncomplete`
   - Add tests for `build()` method availability

4. Update documentation:
   - Document builder pattern usage
   - Show `.build()` final step
   - Explain type safety guarantees

## Next Steps

1. ✅ Confirmed approach with user (Builder pattern with build() method)
2. Implement changes to generate builder struct
3. Update type alias generation
4. Add build() method generation
5. Update all tests to use new pattern
6. Update example files
7. Test thoroughly
