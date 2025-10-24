# Test Suite for Config Proc-Macro

This directory contains comprehensive integration tests for the `#[derive(Config)]` proc-macro.

## Test Files

### 1. `basic_no_incomplete.rs`
**Purpose**: Test structs with NO `#[incomplete]` markers

**What it tests**:
- `{StructName}Complete` type alias is generated
- `{StructName}Incomplete` type alias is NOT generated
- Builder pattern works correctly
- Builder methods can be called in any order

**Example struct**:
```rust
#[derive(Config)]
struct BasicConfig {
    name: String,
    port: u16,
    enabled: bool,
}
```

### 2. `single_incomplete.rs`
**Purpose**: Test structs with a SINGLE `#[incomplete]` marker

**What it tests**:
- Both `{StructName}Complete` and `{StructName}Incomplete` are generated
- Incomplete type correctly represents config without the marked field
- Progressive building from incomplete to complete
- Typestate transitions work correctly

**Example struct**:
```rust
#[derive(Config)]
struct ApiConfig {
    endpoint: String,
    timeout: u64,
    #[incomplete]
    api_key: String,
}
```

### 3. `multiple_incomplete.rs`
**Purpose**: Test structs with MULTIPLE `#[incomplete]` markers

**What it tests**:
- All fields marked `#[incomplete]` become `()` in a single incomplete type alias
- Complete type requires all fields including multiple incomplete ones
- Incomplete type omits all marked fields
- Builder can add incomplete fields in any order

**Example struct**:
```rust
#[derive(Config)]
struct DatabaseConfig {
    host: String,
    port: u16,
    #[incomplete]
    username: String,
    #[incomplete]
    password: String,
}
```

### 4. `dynamic_naming.rs`
**Purpose**: Test that type aliases adapt to struct name

**What it tests**:
- `Foo` generates `FooComplete`, not `ConfigComplete`
- `Bar` generates `BarComplete` and `BarIncomplete`
- Multiple structs in same scope don't conflict
- Type aliases correctly match their source struct names

**Example structs**:
```rust
#[derive(Config)]
struct ServerSettings { ... }  // → ServerSettingsComplete, ServerSettingsIncomplete

#[derive(Config)]
struct AppOptions { ... }       // → AppOptionsComplete

#[derive(Config)]
struct MyCustomStruct { ... }   // → MyCustomStructComplete, MyCustomStructIncomplete
```

### 5. `original_example.rs`
**Purpose**: Test the original example from `config_macro.rs`

**What it tests**:
- Matches the expected behavior from `config_generated.rs`
- `Config` generates `ConfigComplete` and `ConfigIncomplete`
- Typestate pattern works as expected
- Builder methods properly transform types

**Example struct** (matching `config_macro.rs`):
```rust
#[derive(Config)]
struct Config {
    url: mock_types::Url,
    name: String,
    #[incomplete]
    secret: mock_types::Zeroizing<String>,
}
```

## Running the Tests

To run all tests:
```bash
cargo test
```

To run a specific test file:
```bash
cargo test --test basic_no_incomplete
cargo test --test single_incomplete
cargo test --test multiple_incomplete
cargo test --test dynamic_naming
cargo test --test original_example
```

To run a specific test function:
```bash
cargo test test_basic_config_complete_exists
```

## Test Coverage

The test suite covers all requirements from the implementation plan:

- ✅ Basic struct with all required fields (no `#[incomplete]` markers)
- ✅ Struct with single `#[incomplete]` attribute
- ✅ Struct with multiple `#[incomplete]` attributes
- ✅ Dynamic naming (struct named something other than `Config`)
- ✅ Builder pattern compiles correctly
- ✅ Type safety and typestate transitions
- ✅ Generated code matches expected output

## Expected Results

All tests should pass, demonstrating that:
1. The macro correctly parses struct definitions
2. Generic structs are generated with proper type parameters
3. Type aliases are conditionally generated based on `#[incomplete]` markers
4. Type alias names adapt to the source struct name
5. Builder methods enable progressive, type-safe configuration construction
6. The typestate pattern prevents incomplete configurations from being used incorrectly
