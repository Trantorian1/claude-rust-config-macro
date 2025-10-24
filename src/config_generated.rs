struct Config<Url, Name, Secret> {
    url: Url,
    name: Name,
    secret: Secret,
}

pub type ConfigComplete = Config<url::Url, String, zeroize::Zeroizing<String>>;
pub type ConfigIncomplete = Config<url::Url, String, ()>;

impl Config<(), (), ()> {
    pub fn new() -> Self {
        Self {
            url: (),
            name: (),
            secret: (),
        }
    }
}

impl<Url, Name, Secret> Config<Url, Name, Secret> {
    pub fn with_url(self, url: url::Url) -> Config<url::Url, Name, Secret> {
        Config {
            url,
            name: self.name,
            secret: self.secret,
        }
    }

    pub fn with_name(self, name: String) -> Config<Url, String, Secret> {
        Config {
            url: self.url,
            name,
            secret: self.secret,
        }
    }

    pub fn with_secret(
        self,
        secret: zeroize::Zeroizing<String>,
    ) -> Config<Url, Name, zeroize::Zeroizing<String>> {
        Config {
            url: self.url,
            name: self.name,
            secret,
        }
    }
}
