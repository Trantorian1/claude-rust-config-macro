# Builder Derive Macro

A Rust procedural macro that generates type-safe builder patterns using the typestate pattern. This macro automatically creates builder structs with compile-time guarantees that all required fields are set before construction.

## Features

- **Type-Safe Builders**: Generate builder structs with progressive type state
- **Compile-Time Validation**: Ensure all fields are set before calling `build()`
- **Optional Fields**: Mark fields as `#[incomplete]` to create partial configurations
- **Zero Boilerplate**: Automatic builder generation from struct definitions
- **No Name Conflicts**: Generated builders don't interfere with original structs

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
claude-rust-config-macro = "0.1.0"
```

## Usage

### Basic Example

```rust
use claude_rust_config_macro::Builder;

#[derive(Builder)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

fn main() {
    let config = ServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(8080)
        .with_workers(4)
        .build();

    println!("Server running on {}:{}", config.host, config.port);
}
```

### Incomplete Fields

Mark optional fields with `#[incomplete]` to create partial configurations:

```rust
#[derive(Builder)]
struct DatabaseConfig {
    host: String,
    port: u16,
    #[incomplete]
    password: String,
}

fn main() {
    // Create incomplete config without password
    let incomplete: DatabaseConfigIncomplete = DatabaseConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(5432);

    // Later, complete it with password
    let complete = DatabaseConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(5432)
        .with_password("secret".to_string())
        .build();
}
```

## How It Works

For each struct annotated with `#[derive(Builder)]`, the macro generates:

1. **Builder Struct**: `{StructName}Builder<T1, T2, ...>` with generic type parameters
2. **Constructor**: `new()` method returning builder with all fields as `()`
3. **Setter Methods**: `with_{field}()` methods that progressively set fields
4. **Build Method**: `build()` method available only when all fields are concrete types
5. **Incomplete Type Alias**: (Optional) `{StructName}Incomplete` when `#[incomplete]` markers exist

### Generated Code Example

```rust
#[derive(Builder)]
struct Config {
    name: String,
    #[incomplete]
    secret: String,
}
```

Generates:

```rust
// Original struct remains unchanged
struct Config {
    name: String,
    secret: String,
}

// Generated builder
struct ConfigBuilder<Name, Secret> {
    name: Name,
    secret: Secret,
}

// Type alias for incomplete state
pub type ConfigIncomplete = ConfigBuilder<String, ()>;

// Constructor
impl ConfigBuilder<(), ()> {
    pub fn new() -> Self { ... }
}

// Builder methods
impl<Name, Secret> ConfigBuilder<Name, Secret> {
    pub fn with_name(self, name: String) -> ConfigBuilder<String, Secret> { ... }
    pub fn with_secret(self, secret: String) -> ConfigBuilder<Name, String> { ... }
}

// Build method (only available when all fields are concrete)
impl ConfigBuilder<String, String> {
    pub fn build(self) -> Config { ... }
}
```

## Typestate Pattern

The generated builders use the **typestate pattern** to enforce correctness at compile time:

- Each field starts as `()` (unit type)
- Calling `with_field()` replaces the generic with the concrete type
- `build()` method is only available when all generics are concrete types
- Incomplete configurations cannot accidentally be used where complete ones are required

## Requirements

- Rust 2021 edition or later
- Works only with structs that have named fields

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
