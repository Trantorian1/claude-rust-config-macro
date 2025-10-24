// Test: Original example from config_macro.rs
// Verifies that the macro generates code matching config_generated.rs expectations

use claude_rust_config_macro::Config;

// Note: Using mock types since we don't have url and zeroize crates as dependencies
mod mock_types {
    pub struct Url(pub String);
    pub struct Zeroizing<T>(pub T);
}

#[derive(Config)]
struct Config {
    url: mock_types::Url,
    name: String,
    #[incomplete]
    secret: mock_types::Zeroizing<String>,
}

#[test]
fn test_config_build_method() {
    // build() should return Config when all fields are set
    let config: Config = ConfigBuilder::new()
        .with_url(mock_types::Url("https://example.com".to_string()))
        .with_name("MyApp".to_string())
        .with_secret(mock_types::Zeroizing("secret".to_string()))
        .build();

    let _: Config = config;
}

#[test]
fn test_config_incomplete_type_exists() {
    // ConfigIncomplete should exist with secret as ()
    let config: ConfigIncomplete = ConfigBuilder::new()
        .with_url(mock_types::Url("https://example.com".to_string()))
        .with_name("MyApp".to_string());

    let _: ConfigIncomplete = config;
}

#[test]
fn test_config_new_constructor() {
    // new() should return ConfigBuilder<(), (), ()>
    let builder = ConfigBuilder::new();

    // Should be able to call all builder methods and build
    let config = builder
        .with_url(mock_types::Url("https://example.com".to_string()))
        .with_name("MyApp".to_string())
        .with_secret(mock_types::Zeroizing("secret".to_string()))
        .build();

    let _: Config = config;
}

#[test]
fn test_config_builder_methods() {
    // Each builder method should consume self and return new type
    let step1 = ConfigBuilder::new();
    let step2 = step1.with_url(mock_types::Url("https://example.com".to_string()));
    let step3 = step2.with_name("MyApp".to_string());

    // At this point, we have ConfigIncomplete (missing secret)
    let _: ConfigIncomplete = step3;

    // Complete it by adding secret and build
    let step4 = ConfigBuilder::new()
        .with_url(mock_types::Url("https://example.com".to_string()))
        .with_name("MyApp".to_string())
        .with_secret(mock_types::Zeroizing("secret".to_string()))
        .build();

    let _: Config = step4;
}

#[test]
fn test_config_typestate_progression() {
    // Demonstrate the typestate pattern progression
    let incomplete_builder = ConfigBuilder::new()
        .with_url(mock_types::Url("https://api.example.com".to_string()))
        .with_name("Production".to_string());

    // This is incomplete - can be used where ConfigIncomplete is expected
    let incomplete_config: ConfigIncomplete = incomplete_builder;

    // To get complete config, must add secret and call build()
    let complete_config = ConfigBuilder::new()
        .with_url(mock_types::Url("https://api.example.com".to_string()))
        .with_name("Production".to_string())
        .with_secret(mock_types::Zeroizing("prod-secret".to_string()))
        .build();

    let _: Config = complete_config;
    let _: ConfigIncomplete = incomplete_config;
}
