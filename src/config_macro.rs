#[derive(Config)]
struct Config {
    url: url::Url,
    name: String,
    #[incomplete]
    secret: zeroize::Zeroizing<String>,
}
