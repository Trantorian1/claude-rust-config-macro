// Test: Dynamic naming - type aliases should match struct name
// Tests that Foo generates FooComplete/FooIncomplete, not ConfigComplete/ConfigIncomplete

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
    // Should generate ServerSettingsComplete and ServerSettingsIncomplete
    let complete: ServerSettingsComplete = ServerSettings::new()
        .with_address("0.0.0.0:8080".to_string())
        .with_tls_cert("/path/to/cert".to_string());

    let incomplete: ServerSettingsIncomplete = ServerSettings::new()
        .with_address("0.0.0.0:8080".to_string());

    let _: ServerSettingsComplete = complete;
    let _: ServerSettingsIncomplete = incomplete;
}

#[test]
fn test_app_options_naming() {
    // Should generate AppOptionsComplete only (no incomplete markers)
    let complete: AppOptionsComplete = AppOptions::new()
        .with_debug(true)
        .with_log_level("info".to_string());

    let _: AppOptionsComplete = complete;
}

#[test]
fn test_my_custom_struct_naming() {
    // Should generate MyCustomStructComplete and MyCustomStructIncomplete
    let complete: MyCustomStructComplete = MyCustomStruct::new()
        .with_field1("value".to_string())
        .with_field2(42)
        .with_field3(true);

    let incomplete: MyCustomStructIncomplete = MyCustomStruct::new()
        .with_field1("value".to_string())
        .with_field2(42);

    let _: MyCustomStructComplete = complete;
    let _: MyCustomStructIncomplete = incomplete;
}

#[test]
fn test_different_structs_dont_conflict() {
    // Verify that different structs generate different type aliases
    let server: ServerSettingsComplete = ServerSettings::new()
        .with_address("0.0.0.0:8080".to_string())
        .with_tls_cert("/path/to/cert".to_string());

    let app: AppOptionsComplete = AppOptions::new()
        .with_debug(false)
        .with_log_level("debug".to_string());

    let custom: MyCustomStructComplete = MyCustomStruct::new()
        .with_field1("test".to_string())
        .with_field2(100)
        .with_field3(false);

    let _: ServerSettingsComplete = server;
    let _: AppOptionsComplete = app;
    let _: MyCustomStructComplete = custom;
}
