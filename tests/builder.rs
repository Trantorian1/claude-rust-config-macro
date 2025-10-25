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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
struct HttpClient {
    timeout: u64,
    #[incomplete]
    auth_token: String,
}

#[derive(Builder)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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

// ============================================================================
// Test 6: Smart wrapping for Arc<dyn Trait>
// ============================================================================

use std::sync::Arc;

trait LoggerTrait {
    fn log(&self, msg: &str);
}

struct ConsoleLogger {
    prefix: String,
}

impl LoggerTrait for ConsoleLogger {
    fn log(&self, msg: &str) {
        println!("{}: {}", self.prefix, msg);
    }
}

#[derive(Builder)]
struct App {
    name: String,
    logger: Arc<dyn LoggerTrait>,
}

#[test]
fn test_arc_dyn_trait_auto_wrap() {
    // No need to manually wrap with Arc::new!
    let app = AppBuilder::new()
        .with_name("MyApp".to_string())
        .with_logger(ConsoleLogger {
            prefix: "INFO".to_string(),
        })
        .build();

    assert_eq!(app.name, "MyApp");
    app.logger.log("test message");
}

// ============================================================================
// Test 7: Smart wrapping for Box<dyn Trait>
// ============================================================================

trait ProcessorTrait {
    fn process(&self, data: &str) -> String;
}

struct UpperCaseProcessor;

impl ProcessorTrait for UpperCaseProcessor {
    fn process(&self, data: &str) -> String {
        data.to_uppercase()
    }
}

#[derive(Builder)]
struct Pipeline {
    name: String,
    processor: Box<dyn ProcessorTrait>,
}

#[test]
fn test_box_dyn_trait_auto_wrap() {
    // No need to manually wrap with Box::new!
    let pipeline = PipelineBuilder::new()
        .with_name("Transform".to_string())
        .with_processor(UpperCaseProcessor)
        .build();

    assert_eq!(pipeline.name, "Transform");
    assert_eq!(pipeline.processor.process("hello"), "HELLO");
}

// ============================================================================
// Test 8: Multiple trait bounds (Arc<dyn Trait + Send + Sync>)
// ============================================================================

trait HandlerTrait: Send + Sync {
    fn handle(&self, req: &str) -> String;
}

struct EchoHandler;

impl HandlerTrait for EchoHandler {
    fn handle(&self, req: &str) -> String {
        format!("Echo: {}", req)
    }
}

#[derive(Builder)]
struct Server {
    port: u16,
    handler: Arc<dyn HandlerTrait + Send + Sync>,
}

#[test]
fn test_multi_bound_auto_wrap() {
    // Automatically handles Send + Sync bounds
    let server = ServerBuilder::new()
        .with_port(8080)
        .with_handler(EchoHandler)
        .build();

    assert_eq!(server.port, 8080);
    assert_eq!(server.handler.handle("test"), "Echo: test");
}

// ============================================================================
// Test 9: Mixed fields - some wrapped, some not
// ============================================================================

#[derive(Builder)]
#[allow(dead_code)]
struct Service {
    name: String,
    port: u16,
    logger: Arc<dyn LoggerTrait>,
    max_connections: usize,
}

#[test]
fn test_mixed_wrapper_and_regular_fields() {
    let service = ServiceBuilder::new()
        .with_name("API".to_string())
        .with_port(3000)
        .with_logger(ConsoleLogger {
            prefix: "API".to_string(),
        })
        .with_max_connections(100)
        .build();

    assert_eq!(service.name, "API");
    assert_eq!(service.port, 3000);
    assert_eq!(service.max_connections, 100);
}

// ============================================================================
// Test 10: Arc/Box with #[incomplete] marker
// ============================================================================

#[derive(Builder)]
#[allow(dead_code)]
struct Worker {
    id: String,
    #[incomplete]
    processor: Arc<dyn ProcessorTrait>,
}

#[test]
fn test_wrapper_with_incomplete() {
    // Can create incomplete without processor
    let _incomplete: WorkerIncomplete = WorkerBuilder::new().with_id("worker-1".to_string());

    // Can build complete with processor (auto-wrapped)
    let complete = WorkerBuilder::new()
        .with_id("worker-1".to_string())
        .with_processor(UpperCaseProcessor)
        .build();

    assert_eq!(complete.id, "worker-1");
}

// ============================================================================
// Test 11: Basic default field
// ============================================================================

#[derive(Builder)]
struct WebServerConfig {
    host: String,
    port: u16,
    #[default(4)]
    workers: usize,
}

#[test]
fn test_basic_default_field() {
    // Can build without setting default field
    let config = WebServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(8080)
        .build();

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8080);
    assert_eq!(config.workers, 4); // Uses default value
}

#[test]
fn test_override_default_field() {
    // Can override default value with mutable consuming setter
    let config = WebServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_port(8080)
        .with_workers(8) // Override default
        .build();

    assert_eq!(config.workers, 8);
}

#[test]
fn test_default_field_chaining() {
    // Mutable consuming pattern allows chaining
    let config = WebServerConfigBuilder::new()
        .with_host("localhost".to_string())
        .with_workers(16) // Can set before or after required fields
        .with_port(8080)
        .build();

    assert_eq!(config.workers, 16);
}

// ============================================================================
// Test 12: Multiple default fields
// ============================================================================

#[derive(Builder)]
struct ApiConfig {
    endpoint: String,
    #[default(30)]
    timeout: u64,
    #[default(3)]
    retries: u32,
    #[default(true)]
    use_tls: bool,
}

#[test]
fn test_multiple_defaults() {
    // Can build with only required field
    let config = ApiConfigBuilder::new()
        .with_endpoint("https://api.example.com".to_string())
        .build();

    assert_eq!(config.endpoint, "https://api.example.com");
    assert_eq!(config.timeout, 30);
    assert_eq!(config.retries, 3);
    assert_eq!(config.use_tls, true);
}

#[test]
fn test_partial_override_defaults() {
    // Can override some defaults while keeping others
    let config = ApiConfigBuilder::new()
        .with_endpoint("https://api.example.com".to_string())
        .with_timeout(60) // Override this one
        .with_retries(5)  // And this one
        .build();

    assert_eq!(config.timeout, 60);
    assert_eq!(config.retries, 5);
    assert_eq!(config.use_tls, true); // Keeps default
}

#[test]
fn test_multiple_overrides_order() {
    // Can override in any order
    let config = ApiConfigBuilder::new()
        .with_timeout(45)
        .with_endpoint("https://api.example.com".to_string())
        .with_use_tls(false)
        .build();

    assert_eq!(config.timeout, 45);
    assert_eq!(config.use_tls, false);
}

// ============================================================================
// Test 13: Default with complex types
// ============================================================================

#[derive(Builder)]
struct StorageConfig {
    path: String,
    #[default(vec![])]
    exclude_patterns: Vec<String>,
    #[default(None)]
    cache_dir: Option<String>,
}

#[test]
fn test_default_complex_types() {
    let config = StorageConfigBuilder::new()
        .with_path("/data".to_string())
        .build();

    assert_eq!(config.path, "/data");
    assert!(config.exclude_patterns.is_empty());
    assert!(config.cache_dir.is_none());
}

#[test]
fn test_override_default_complex_types() {
    let config = StorageConfigBuilder::new()
        .with_path("/data".to_string())
        .with_exclude_patterns(vec!["*.tmp".to_string(), "*.log".to_string()])
        .with_cache_dir(Some("/cache".to_string()))
        .build();

    assert_eq!(config.exclude_patterns.len(), 2);
    assert_eq!(config.cache_dir, Some("/cache".to_string()));
}

// ============================================================================
// Test 14: Default with Arc smart wrapping
// ============================================================================

trait FormatterTrait {
    fn format(&self, text: &str) -> String;
}

struct JsonFormatter;

impl FormatterTrait for JsonFormatter {
    fn format(&self, text: &str) -> String {
        format!(r#"{{"text": "{}"}}"#, text)
    }
}

struct DefaultFormatter;

impl FormatterTrait for DefaultFormatter {
    fn format(&self, text: &str) -> String {
        text.to_string()
    }
}

#[derive(Builder)]
struct OutputConfig {
    destination: String,
    #[default(Arc::new(DefaultFormatter))]
    formatter: Arc<dyn FormatterTrait>,
}

#[test]
fn test_default_arc_wrapper() {
    let config = OutputConfigBuilder::new()
        .with_destination("stdout".to_string())
        .build();

    assert_eq!(config.destination, "stdout");
    assert_eq!(config.formatter.format("test"), "test");
}

#[test]
fn test_override_default_arc_wrapper() {
    // Override default formatter with smart wrapping
    let config = OutputConfigBuilder::new()
        .with_destination("stdout".to_string())
        .with_formatter(JsonFormatter)
        .build();

    assert_eq!(config.formatter.format("test"), r#"{"text": "test"}"#);
}

// ============================================================================
// Test 15: Mixing default fields with incomplete fields
// ============================================================================

#[derive(Builder)]
struct SecurityConfig {
    #[incomplete]
    api_key: String,
    #[default("info")]
    log_level: &'static str,
    #[default(true)]
    verify_ssl: bool,
}

#[test]
fn test_default_with_incomplete_fields() {
    // Can build with only api_key (defaults are optional)
    let config = SecurityConfigBuilder::new()
        .with_api_key("key123".to_string())
        .build();

    assert_eq!(config.api_key, "key123");
    assert_eq!(config.log_level, "info");
    assert_eq!(config.verify_ssl, true);
}

#[test]
fn test_incomplete_with_defaults() {
    // Can create incomplete without api_key (defaults are initialized)
    let _incomplete: SecurityConfigIncomplete = SecurityConfigBuilder::new();

    // Must provide api_key to build
    let complete = SecurityConfigBuilder::new()
        .with_api_key("key123".to_string())
        .build();

    assert_eq!(complete.api_key, "key123");
}

#[test]
fn test_override_defaults_with_incomplete() {
    // Can override default values even with incomplete fields
    let config = SecurityConfigBuilder::new()
        .with_api_key("key123".to_string())
        .with_log_level("debug")
        .with_verify_ssl(false)
        .build();

    assert_eq!(config.api_key, "key123");
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.verify_ssl, false);
}

// ============================================================================
// Test 16: All default fields
// ============================================================================

#[derive(Builder)]
struct ThemeConfig {
    #[default("dark")]
    mode: &'static str,
    #[default(14)]
    font_size: u8,
    #[default(true)]
    syntax_highlighting: bool,
}

#[test]
fn test_all_default_fields() {
    // Can build without setting any fields
    let config = ThemeConfigBuilder::new().build();

    assert_eq!(config.mode, "dark");
    assert_eq!(config.font_size, 14);
    assert_eq!(config.syntax_highlighting, true);
}

#[test]
fn test_all_defaults_with_overrides() {
    let config = ThemeConfigBuilder::new()
        .with_mode("light")
        .with_font_size(16)
        .build();

    assert_eq!(config.mode, "light");
    assert_eq!(config.font_size, 16);
    assert_eq!(config.syntax_highlighting, true); // Keeps default
}
