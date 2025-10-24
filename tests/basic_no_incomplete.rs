// Test: Struct with no #[incomplete] markers
// Should generate only {StructName}Complete, not {StructName}Incomplete

use claude_rust_config_macro::Config;

#[derive(Config)]
struct BasicConfig {
    name: String,
    port: u16,
    enabled: bool,
}

#[test]
fn test_basic_config_complete_exists() {
    // BasicConfigComplete should be generated
    let _config: BasicConfigComplete = BasicConfig::new()
        .with_name("test".to_string())
        .with_port(8080)
        .with_enabled(true);
}

#[test]
fn test_basic_config_builder_pattern() {
    // Test that builder pattern works correctly
    let config = BasicConfig::new()
        .with_name("MyApp".to_string())
        .with_port(3000)
        .with_enabled(false);

    // Should be type BasicConfigComplete
    let _: BasicConfigComplete = config;
}

#[test]
fn test_basic_config_builder_order_independence() {
    // Builder methods should work in any order
    let config1 = BasicConfig::new()
        .with_name("App1".to_string())
        .with_port(8080)
        .with_enabled(true);

    let config2 = BasicConfig::new()
        .with_enabled(true)
        .with_name("App2".to_string())
        .with_port(8080);

    let _: BasicConfigComplete = config1;
    let _: BasicConfigComplete = config2;
}

// This test verifies that BasicConfigIncomplete is NOT generated
// If it were generated, this would compile, but it should fail
// Uncomment to verify:
// #[test]
// fn test_basic_config_incomplete_does_not_exist() {
//     let _: BasicConfigIncomplete = BasicConfig::new();
// }
