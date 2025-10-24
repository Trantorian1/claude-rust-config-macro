// Test: Struct with single #[incomplete] marker
// Should generate both {StructName}Complete and {StructName}Incomplete

use claude_rust_config_macro::Config;

#[derive(Config)]
struct ApiConfig {
    endpoint: String,
    timeout: u64,
    #[incomplete]
    api_key: String,
}

#[test]
fn test_api_config_complete_exists() {
    // ApiConfigComplete should be generated
    let config: ApiConfigComplete = ApiConfig::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30)
        .with_api_key("secret-key".to_string());

    let _: ApiConfigComplete = config;
}

#[test]
fn test_api_config_incomplete_exists() {
    // ApiConfigIncomplete should be generated (without api_key)
    let config: ApiConfigIncomplete = ApiConfig::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30);

    let _: ApiConfigIncomplete = config;
}

#[test]
fn test_api_config_builder_to_complete() {
    // Start with incomplete, add api_key to get complete
    let incomplete = ApiConfig::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30);

    let _: ApiConfigIncomplete = incomplete;

    // Now complete it
    let complete = ApiConfig::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30)
        .with_api_key("secret".to_string());

    let _: ApiConfigComplete = complete;
}

#[test]
fn test_api_config_progressive_building() {
    // Test progressive type state changes
    let step1 = ApiConfig::new();
    let step2 = step1.with_endpoint("https://api.example.com".to_string());
    let step3 = step2.with_timeout(30);

    // At this point, we have ApiConfigIncomplete
    let _: ApiConfigIncomplete = step3;
}
