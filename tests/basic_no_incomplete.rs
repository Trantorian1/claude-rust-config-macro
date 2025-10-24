// Test: Struct with no #[incomplete] markers
// Should generate builder but NO {StructName}Incomplete type alias

use claude_rust_config_macro::Config;

#[derive(Config)]
struct BasicConfig {
    name: String,
    port: u16,
    enabled: bool,
}

#[test]
fn test_basic_config_build_method() {
    // Builder pattern with build() returns BasicConfig
    let config: BasicConfig = BasicConfigBuilder::new()
        .with_name("test".to_string())
        .with_port(8080)
        .with_enabled(true)
        .build();

    let _: BasicConfig = config;
}

#[test]
fn test_basic_config_builder_pattern() {
    // Test that builder pattern works correctly
    let config = BasicConfigBuilder::new()
        .with_name("MyApp".to_string())
        .with_port(3000)
        .with_enabled(false)
        .build();

    // Should be type BasicConfig
    let _: BasicConfig = config;
}

#[test]
fn test_basic_config_builder_order_independence() {
    // Builder methods should work in any order
    let config1 = BasicConfigBuilder::new()
        .with_name("App1".to_string())
        .with_port(8080)
        .with_enabled(true)
        .build();

    let config2 = BasicConfigBuilder::new()
        .with_enabled(true)
        .with_name("App2".to_string())
        .with_port(8080)
        .build();

    let _: BasicConfig = config1;
    let _: BasicConfig = config2;
}

// This test verifies that BasicConfigIncomplete is NOT generated
// If it were generated, this would compile, but it should fail
// Uncomment to verify:
// #[test]
// fn test_basic_config_incomplete_does_not_exist() {
//     let _: BasicConfigIncomplete = BasicConfigBuilder::new();
// }
