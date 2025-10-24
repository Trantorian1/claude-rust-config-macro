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
fn test_config_complete_type_exists() {
    // ConfigComplete should exist and include all three fields
    let config: ConfigComplete = Config::new()
        .with_url(mock_types::Url("https://example.com".to_string()))
        .with_name("MyApp".to_string())
        .with_secret(mock_types::Zeroizing("secret".to_string()));

    let _: ConfigComplete = config;
}

#[test]
fn test_config_incomplete_type_exists() {
    // ConfigIncomplete should exist with secret as ()
    let config: ConfigIncomplete = Config::new()
        .with_url(mock_types::Url("https://example.com".to_string()))
        .with_name("MyApp".to_string());

    let _: ConfigIncomplete = config;
}

#[test]
fn test_config_new_constructor() {
    // new() should return Config<(), (), ()>
    let config = Config::new();

    // Should be able to call all builder methods
    let config = config
        .with_url(mock_types::Url("https://example.com".to_string()))
        .with_name("MyApp".to_string())
        .with_secret(mock_types::Zeroizing("secret".to_string()));

    let _: ConfigComplete = config;
}

#[test]
fn test_config_builder_methods() {
    // Each builder method should consume self and return new type
    let step1 = Config::new();
    let step2 = step1.with_url(mock_types::Url("https://example.com".to_string()));
    let step3 = step2.with_name("MyApp".to_string());

    // At this point, we have ConfigIncomplete (missing secret)
    let _: ConfigIncomplete = step3;

    // Complete it by adding secret
    let step4 = Config::new()
        .with_url(mock_types::Url("https://example.com".to_string()))
        .with_name("MyApp".to_string())
        .with_secret(mock_types::Zeroizing("secret".to_string()));

    let _: ConfigComplete = step4;
}

#[test]
fn test_config_typestate_progression() {
    // Demonstrate the typestate pattern progression
    let config = Config::new()
        .with_url(mock_types::Url("https://api.example.com".to_string()))
        .with_name("Production".to_string());

    // This is incomplete - can be used where ConfigIncomplete is expected
    let incomplete_config: ConfigIncomplete = config;

    // To get complete config, must add secret
    let complete_config = Config::new()
        .with_url(mock_types::Url("https://api.example.com".to_string()))
        .with_name("Production".to_string())
        .with_secret(mock_types::Zeroizing("prod-secret".to_string()));

    let _: ConfigComplete = complete_config;
    let _: ConfigIncomplete = incomplete_config;
}
