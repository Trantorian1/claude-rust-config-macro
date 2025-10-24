// Test: Struct with multiple #[incomplete] markers
// All marked fields should become () in a single {StructName}Incomplete alias

use claude_rust_config_macro::Config;

#[derive(Config)]
struct DatabaseConfig {
    host: String,
    port: u16,
    #[incomplete]
    username: String,
    #[incomplete]
    password: String,
}

#[test]
fn test_database_config_complete_all_fields() {
    // DatabaseConfigComplete should require all fields
    let config: DatabaseConfigComplete = DatabaseConfig::new()
        .with_host("localhost".to_string())
        .with_port(5432)
        .with_username("admin".to_string())
        .with_password("secret".to_string());

    let _: DatabaseConfigComplete = config;
}

#[test]
fn test_database_config_incomplete_without_credentials() {
    // DatabaseConfigIncomplete should be available without username and password
    let config: DatabaseConfigIncomplete = DatabaseConfig::new()
        .with_host("localhost".to_string())
        .with_port(5432);

    let _: DatabaseConfigIncomplete = config;
}

#[test]
fn test_database_config_partial_incomplete() {
    // Even with one credential, it's not complete yet
    let config1 = DatabaseConfig::new()
        .with_host("localhost".to_string())
        .with_port(5432)
        .with_username("admin".to_string());

    // This is neither complete nor the "pure" incomplete state
    // But we can verify it's not DatabaseConfigComplete
    // and we can still add the password
    let config2 = config1.with_password("secret".to_string());

    let _: DatabaseConfigComplete = config2;
}

#[test]
fn test_database_config_builder_flexibility() {
    // Can build in different orders
    let config = DatabaseConfig::new()
        .with_password("secret".to_string())
        .with_username("admin".to_string())
        .with_host("localhost".to_string())
        .with_port(5432);

    let _: DatabaseConfigComplete = config;
}
