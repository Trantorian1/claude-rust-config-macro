// Original Config struct remains untouched by the macro
struct Config {
    url: url::Url,
    name: String,
    secret: zeroize::Zeroizing<String>,
}

// Macro generates ConfigBuilder struct
struct ConfigBuilder<Url, Name, Secret> {
    url: Url,
    name: Name,
    secret: Secret,
}

// Type alias for incomplete state (only generated because #[incomplete] markers exist)
pub type ConfigIncomplete = ConfigBuilder<url::Url, String, ()>;

// Constructor returns builder with all () fields
impl ConfigBuilder<(), (), ()> {
    pub fn new() -> Self {
        Self {
            url: (),
            name: (),
            secret: (),
        }
    }
}

// Builder methods for progressive construction
impl<Url, Name, Secret> ConfigBuilder<Url, Name, Secret> {
    pub fn with_url(self, url: url::Url) -> ConfigBuilder<url::Url, Name, Secret> {
        ConfigBuilder {
            url,
            name: self.name,
            secret: self.secret,
        }
    }

    pub fn with_name(self, name: String) -> ConfigBuilder<Url, String, Secret> {
        ConfigBuilder {
            url: self.url,
            name,
            secret: self.secret,
        }
    }

    pub fn with_secret(
        self,
        secret: zeroize::Zeroizing<String>,
    ) -> ConfigBuilder<Url, Name, zeroize::Zeroizing<String>> {
        ConfigBuilder {
            url: self.url,
            name: self.name,
            secret,
        }
    }
}

// build() method - only available when all fields are concrete
impl ConfigBuilder<url::Url, String, zeroize::Zeroizing<String>> {
    pub fn build(self) -> Config {
        Config {
            url: self.url,
            name: self.name,
            secret: self.secret,
        }
    }
}
