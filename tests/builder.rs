use claude_rust_config_macro::Builder;

// Test 1: Basic struct with no #[incomplete] markers
#[derive(Builder)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

#[test]
fn test_basic_builder() {
    let config: ServerConfig = ServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(8080)
        .with_workers(4)
        .build();

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8080);
    assert_eq!(config.workers, 4);
}

#[test]
fn test_builder_order_independence() {
    let config1 = ServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(8080)
        .with_workers(4)
        .build();

    let config2 = ServerConfigBuilder::new()
        .with_workers(4)
        .with_port(8080)
        .with_host("localhost".to_string())
        .build();

    assert_eq!(config1.host, config2.host);
    assert_eq!(config1.port, config2.port);
}

// Test 2: Struct with single #[incomplete] marker
#[derive(Builder)]
struct DatabaseConfig {
    host: String,
    port: u16,
    #[incomplete]
    password: String,
}

#[test]
fn test_incomplete_single_field() {
    // Can create incomplete config without password
    let _incomplete: DatabaseConfigIncomplete = DatabaseConfigBuilder::new()
        .with_host("db.example.com".to_string())
        .with_port(5432);

    // Can build complete config with password
    let complete: DatabaseConfig = DatabaseConfigBuilder::new()
        .with_host("db.example.com".to_string())
        .with_port(5432)
        .with_password("secret".to_string())
        .build();

    assert_eq!(complete.password, "secret");
}

#[test]
fn test_typestate_progression() {
    let step1 = DatabaseConfigBuilder::new();
    let step2 = step1.with_host("db.example.com".to_string());
    let step3 = step2.with_port(5432);

    // At this point, we have DatabaseConfigIncomplete
    let _incomplete: DatabaseConfigIncomplete = step3;
}

// Test 3: Struct with multiple #[incomplete] markers
#[derive(Builder)]
struct AppConfig {
    name: String,
    version: String,
    #[incomplete]
    api_key: String,
    #[incomplete]
    secret_token: String,
}

#[test]
fn test_incomplete_multiple_fields() {
    // Can create incomplete config without both secrets
    let _incomplete: AppConfigIncomplete = AppConfigBuilder::new()
        .with_name("MyApp".to_string())
        .with_version("1.0.0".to_string());

    // Must provide both secrets to build
    let complete: AppConfig = AppConfigBuilder::new()
        .with_name("MyApp".to_string())
        .with_version("1.0.0".to_string())
        .with_api_key("key123".to_string())
        .with_secret_token("token456".to_string())
        .build();

    assert_eq!(complete.api_key, "key123");
    assert_eq!(complete.secret_token, "token456");
}

#[test]
fn test_partial_incomplete() {
    // Even with one secret, still need both to build
    let builder = AppConfigBuilder::new()
        .with_name("MyApp".to_string())
        .with_version("1.0.0".to_string())
        .with_api_key("key123".to_string());

    // Add second secret and build
    let complete = builder.with_secret_token("token456".to_string()).build();

    assert_eq!(complete.name, "MyApp");
}

// Test 4: Dynamic naming - different struct names
#[derive(Builder)]
struct HttpClient {
    timeout: u64,
    #[incomplete]
    auth_token: String,
}

#[derive(Builder)]
struct Logger {
    level: String,
    output: String,
}

#[test]
fn test_dynamic_naming() {
    // Each struct gets its own builder name
    let client: HttpClient = HttpClientBuilder::new()
        .with_timeout(30)
        .with_auth_token("token".to_string())
        .build();

    let logger: Logger = LoggerBuilder::new()
        .with_level("INFO".to_string())
        .with_output("stdout".to_string())
        .build();

    assert_eq!(client.timeout, 30);
    assert_eq!(logger.level, "INFO");
}

#[test]
fn test_incomplete_type_alias_naming() {
    // Incomplete type alias matches struct name
    let _incomplete: HttpClientIncomplete = HttpClientBuilder::new().with_timeout(30);

    // Logger has no incomplete markers, so no LoggerIncomplete type
    // This would fail to compile:
    // let _incomplete: LoggerIncomplete = LoggerBuilder::new();
}

// Test 5: Complex types
#[derive(Builder)]
struct CacheConfig {
    max_size: usize,
    ttl_seconds: u64,
    tags: Vec<String>,
    #[incomplete]
    redis_url: Option<String>,
}

#[test]
fn test_complex_field_types() {
    let config: CacheConfig = CacheConfigBuilder::new()
        .with_max_size(1000)
        .with_ttl_seconds(3600)
        .with_tags(vec!["cache".to_string(), "prod".to_string()])
        .with_redis_url(Some("redis://localhost".to_string()))
        .build();

    assert_eq!(config.max_size, 1000);
    assert_eq!(config.tags.len(), 2);
    assert!(config.redis_url.is_some());
}
