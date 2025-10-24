// Test: Struct with single #[incomplete] marker
// Should generate builder with build() method and {StructName}Incomplete type alias

use claude_rust_config_macro::Config;

#[derive(Config)]
struct ApiConfig {
    endpoint: String,
    timeout: u64,
    #[incomplete]
    api_key: String,
}

#[test]
fn test_api_config_build_method() {
    // build() returns ApiConfig when all fields are set
    let config: ApiConfig = ApiConfigBuilder::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30)
        .with_api_key("secret-key".to_string())
        .build();

    let _: ApiConfig = config;
}

#[test]
fn test_api_config_incomplete_exists() {
    // ApiConfigIncomplete should be generated (without api_key)
    let config: ApiConfigIncomplete = ApiConfigBuilder::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30);

    let _: ApiConfigIncomplete = config;
}

#[test]
fn test_api_config_builder_to_complete() {
    // Start with incomplete, add api_key to call build()
    let incomplete = ApiConfigBuilder::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30);

    let _: ApiConfigIncomplete = incomplete;

    // Now complete it and build
    let complete = ApiConfigBuilder::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30)
        .with_api_key("secret".to_string())
        .build();

    let _: ApiConfig = complete;
}

#[test]
fn test_api_config_progressive_building() {
    // Test progressive type state changes
    let step1 = ApiConfigBuilder::new();
    let step2 = step1.with_endpoint("https://api.example.com".to_string());
    let step3 = step2.with_timeout(30);

    // At this point, we have ApiConfigIncomplete
    let _: ApiConfigIncomplete = step3;
}

#[test]
fn test_api_config_build_only_when_complete() {
    // This demonstrates that build() is only available on fully concrete types
    let complete_builder = ApiConfigBuilder::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(30)
        .with_api_key("secret".to_string());

    // build() method is available here
    let _config: ApiConfig = complete_builder.build();

    // Incomplete builder does not have build() method
    // Uncomment to verify compilation fails:
    // let incomplete_builder = ApiConfigBuilder::new()
    //     .with_endpoint("https://api.example.com".to_string())
    //     .with_timeout(30);
    // let _config: ApiConfig = incomplete_builder.build(); // Won't compile!
}
