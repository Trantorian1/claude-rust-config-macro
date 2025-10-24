// Test: Dynamic naming - builders and type aliases should match struct name
// Tests that Foo generates FooBuilder/FooIncomplete, not ConfigBuilder/ConfigIncomplete

use claude_rust_config_macro::Config;

#[derive(Config)]
struct ServerSettings {
    address: String,
    #[incomplete]
    tls_cert: String,
}

#[derive(Config)]
struct AppOptions {
    debug: bool,
    log_level: String,
}

#[derive(Config)]
struct MyCustomStruct {
    field1: String,
    field2: i32,
    #[incomplete]
    field3: bool,
}

#[test]
fn test_server_settings_naming() {
    // Should generate ServerSettingsBuilder and ServerSettingsIncomplete
    let complete: ServerSettings = ServerSettingsBuilder::new()
        .with_address("0.0.0.0:8080".to_string())
        .with_tls_cert("/path/to/cert".to_string())
        .build();

    let incomplete: ServerSettingsIncomplete = ServerSettingsBuilder::new()
        .with_address("0.0.0.0:8080".to_string());

    let _: ServerSettings = complete;
    let _: ServerSettingsIncomplete = incomplete;
}

#[test]
fn test_app_options_naming() {
    // Should generate AppOptionsBuilder (no incomplete type alias since no markers)
    let complete: AppOptions = AppOptionsBuilder::new()
        .with_debug(true)
        .with_log_level("info".to_string())
        .build();

    let _: AppOptions = complete;
}

#[test]
fn test_my_custom_struct_naming() {
    // Should generate MyCustomStructBuilder and MyCustomStructIncomplete
    let complete: MyCustomStruct = MyCustomStructBuilder::new()
        .with_field1("value".to_string())
        .with_field2(42)
        .with_field3(true)
        .build();

    let incomplete: MyCustomStructIncomplete = MyCustomStructBuilder::new()
        .with_field1("value".to_string())
        .with_field2(42);

    let _: MyCustomStruct = complete;
    let _: MyCustomStructIncomplete = incomplete;
}

#[test]
fn test_different_structs_dont_conflict() {
    // Verify that different structs generate different builders
    let server: ServerSettings = ServerSettingsBuilder::new()
        .with_address("0.0.0.0:8080".to_string())
        .with_tls_cert("/path/to/cert".to_string())
        .build();

    let app: AppOptions = AppOptionsBuilder::new()
        .with_debug(false)
        .with_log_level("debug".to_string())
        .build();

    let custom: MyCustomStruct = MyCustomStructBuilder::new()
        .with_field1("test".to_string())
        .with_field2(100)
        .with_field3(false)
        .build();

    let _: ServerSettings = server;
    let _: AppOptions = app;
    let _: MyCustomStruct = custom;
}
