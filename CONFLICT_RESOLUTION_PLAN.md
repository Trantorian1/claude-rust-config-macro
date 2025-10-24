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

## Recommended Solution: Option 1 (Attribute Macro)

### Why Attribute Macro?

1. **Correct semantics**: We want to *transform* the struct definition, not just derive traits
2. **Matches expectations**: The `config_generated.rs` shows only the generic struct, not the original
3. **Clean output**: Only one struct definition exists
4. **Standard pattern**: Other builder/typestate macros use this approach

### Implementation Changes Required

#### 1. Change Macro Type
**Before** (in `src/lib.rs`):
```rust
#[proc_macro_derive(Config, attributes(incomplete))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    // ...
}
```

**After**:
```rust
#[proc_macro_attribute]
pub fn config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse item as DeriveInput
    // Generate generic struct + type aliases + impls
    // Return complete replacement
}
```

#### 2. Update Syntax
**Before**:
```rust
#[derive(Config)]
struct Config { ... }
```

**After**:
```rust
#[config]
struct Config { ... }
```

#### 3. Update Generated Code
The generated code remains the same, but now it *replaces* the original instead of adding to it.

### Migration Path

1. Update `src/lib.rs`:
   - Change from `proc_macro_derive` to `proc_macro_attribute`
   - Update function signature to accept attribute tokens
   - Keep same parsing and generation logic

2. Update example files:
   - Change `#[derive(Config)]` to `#[config]` in `config_macro.rs`

3. Update tests:
   - Change all `#[derive(Config)]` to `#[config]`
   - Verify tests still pass

4. Update documentation:
   - Update README and implementation plan
   - Show correct syntax

## Alternative: Minimal Breaking Change (Option 2 Variant)

If changing to attribute macro is not desired, we could:

1. Keep derive macro
2. Generate `{StructName}Internal<...>` (hidden generic struct)
3. Keep original struct but mark with `#[doc(hidden)]` or deprecation
4. Use type aliases for everything:
   ```rust
   #[doc(hidden)]
   struct ConfigInternal<Url, Name, Secret> { ... }

   pub type ConfigComplete = ConfigInternal<url::Url, String, Zeroizing<String>>;
   pub type ConfigIncomplete = ConfigInternal<url::Url, String, ()>;
   ```

However, this is less clean and still leaves the conflict.

## Recommendation

**Use Option 1: Attribute Macro**

This is the cleanest, most correct solution that:
- Eliminates the conflict entirely
- Matches the expected output
- Provides better semantics (transformation vs. derivation)
- Follows established patterns in the Rust ecosystem

The only downside is the syntax change from `#[derive(Config)]` to `#[config]`, but this is a worthwhile trade-off for correctness.

## Next Steps

1. Confirm approach with user
2. Implement attribute macro version
3. Update all examples and tests
4. Document the change
5. Test thoroughly
