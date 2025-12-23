#[cfg(feature = "server")]
pub mod server {
    use base64::prelude::*;
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::path::PathBuf;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ConfigError {
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
        #[error("TOML parsing error: {0}")]
        Toml(#[from] toml::de::Error),
        #[error("URL parsing error: {0}")]
        Url(#[from] url::ParseError),
        #[error("Base64 decoding error: {0}")]
        Base64(#[from] base64::DecodeError),
        #[error("Validation error: {0}")]
        Validation(String),
        #[error("Missing required configuration: {0}")]
        Missing(String),
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct EmailCredentials {
        pub username: String,
        #[serde(skip_serializing)]
        pub password: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct EmailConfig {
        pub smtp_server: String,
        pub credentials: EmailCredentials,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Config {
        #[serde(
            serialize_with = "serialize_cookie_key",
            deserialize_with = "deserialize_cookie_key"
        )]
        pub cookies_secret: tower_cookies::Key,
        #[serde(
            serialize_with = "serialize_totp_key",
            deserialize_with = "deserialize_totp_key"
        )]
        pub totp_encryption_key: aes_gcm::Key<aes_gcm::Aes256Gcm>,
        pub email_config: Option<EmailConfig>,
        #[serde(default = "default_base_url")]
        pub base_url: url::Url,
        #[serde(default)]
        pub family_name: Option<String>,
        #[serde(default)]
        pub family_description: Option<String>,
        #[serde(default = "default_port")]
        pub port: u16,
        #[serde(default = "default_log_level")]
        pub log_level: String,
        #[serde(default)]
        pub public: bool,
    }

    fn default_port() -> u16 {
        8080
    }

    fn default_log_level() -> String {
        "info".to_string()
    }

    fn default_base_url() -> url::Url {
        "http://localhost:8080".parse().unwrap()
    }

    fn serialize_totp_key<S>(
        key: &aes_gcm::Key<aes_gcm::Aes256Gcm>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded = BASE64_STANDARD.encode(key.as_slice());
        serializer.serialize_str(&encoded)
    }

    fn deserialize_totp_key<'de, D>(
        deserializer: D,
    ) -> Result<aes_gcm::Key<aes_gcm::Aes256Gcm>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_exact_iter(
            BASE64_STANDARD
                .decode(encoded)
                .map_err(serde::de::Error::custom)?,
        )
        .ok_or(serde::de::Error::custom("Failed to deserialize totp key"))
    }

    fn serialize_cookie_key<S>(key: &tower_cookies::Key, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded = BASE64_STANDARD.encode(key.master());
        serializer.serialize_str(&encoded)
    }

    fn deserialize_cookie_key<'de, D>(deserializer: D) -> Result<tower_cookies::Key, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        Ok(tower_cookies::Key::from(
            &BASE64_STANDARD
                .decode(encoded)
                .map_err(serde::de::Error::custom)?,
        ))
    }

    impl Config {
        /// Load configuration with the following precedence (highest to lowest):
        /// 1. Environment variables
        /// 2. TOML configuration file
        /// 3. Defaults (where applicable)
        pub fn load_config() -> Result<Self, ConfigError> {
            tracing::info!("Loading configuration...");
            let mut config = Self::load_from_toml().unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to load TOML config: {}. Using defaults where possible.",
                    e
                );
                Self::default()
            });
            config.apply_env_overrides()?;
            config.validate()?;
            tracing::info!("Configuration loaded successfully");
            Ok(config)
        }

        fn load_from_toml() -> Result<Self, ConfigError> {
            let config_path = Self::get_config_path();
            tracing::debug!("Loading config from: {:?}", config_path);
            let config_content = std::fs::read_to_string(&config_path).map_err(|e| {
                tracing::error!("Failed to read config file {:?}: {}", config_path, e);
                e
            })?;
            let config: Self = toml::from_str(&config_content)?;
            Ok(config)
        }

        fn get_config_path() -> PathBuf {
            let config_dir = env::var("SHAJARAH_CONFIG_PATH").unwrap_or_else(|_| ".".to_string());
            let config_name =
                env::var("SHAJARAH_CONFIG_FILE").unwrap_or_else(|_| "config.toml".to_string());
            let mut path = PathBuf::from(config_dir);
            path.push(config_name);
            path
        }

        fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
            if let Ok(secret_b64) = env::var("SHAJARAH_COOKIE_SECRET") {
                tracing::debug!("Using cookie secret from environment");
                self.cookies_secret =
                    tower_cookies::Key::from(&BASE64_STANDARD.decode(secret_b64)?);
            }
            if let Ok(key_b64) = env::var("SHAJARAH_TOTP_KEY") {
                tracing::debug!("Using TOTP encryption key from environment");
                self.totp_encryption_key = aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_exact_iter(
                    BASE64_STANDARD.decode(key_b64)?,
                )
                .expect("Valid AES256GCM key");
            }
            if let Ok(base_url) = env::var("SHAJARAH_BASE_URL") {
                tracing::debug!("Using base URL from environment: {}", base_url);
                self.base_url = base_url.parse()?;
            }
            if let Ok(port_str) = env::var("SHAJARAH_PORT") {
                self.port = port_str
                    .parse()
                    .map_err(|_| ConfigError::Validation("Invalid port number".to_string()))?;
            }
            if let Ok(log_level) = env::var("SHAJARAH_LOG_LEVEL") {
                self.log_level = log_level;
            }
            if let Ok(family_name) = env::var("SHAJARAH_FAMILY_NAME") {
                tracing::debug!("Using family name from environment: {}", family_name);
                self.family_name = Some(family_name);
            }
            self.apply_email_env_overrides()?;
            Ok(())
        }

        fn apply_email_env_overrides(&mut self) -> Result<(), ConfigError> {
            let smtp_server = env::var("SHAJARAH_SMTP_SERVER").ok();
            let email_username = env::var("SHAJARAH_EMAIL_USERNAME").ok();
            let email_password = env::var("SHAJARAH_EMAIL_PASSWORD").ok();
            if smtp_server.is_some() || email_username.is_some() || email_password.is_some() {
                let mut email_config = self.email_config.clone().unwrap_or_else(|| {
                    tracing::debug!("Creating email config from environment variables");
                    EmailConfig {
                        smtp_server: String::new(),
                        credentials: EmailCredentials {
                            username: String::new(),
                            password: String::new(),
                        },
                    }
                });
                if let Some(server) = smtp_server {
                    email_config.smtp_server = server;
                }
                if let Some(username) = email_username {
                    email_config.credentials.username = username;
                }
                if let Some(password) = email_password {
                    email_config.credentials.password = password;
                }
                self.email_config = Some(email_config);
            }
            Ok(())
        }

        fn validate(&self) -> Result<(), ConfigError> {
            if self.totp_encryption_key.is_empty() {
                return Err(ConfigError::Missing(
                    "totp_encryption_key is required (set SHAJARAH_TOTP_KEY or add to config file)"
                        .to_string(),
                ));
            }
            if self.totp_encryption_key.len() != 32 {
                return Err(ConfigError::Validation(
                    "totp_encryption_key should be exactly 32 bytes (256 bits)".to_string(),
                ));
            }
            if let Some(ref email_config) = self.email_config {
                if email_config.smtp_server.is_empty() {
                    return Err(ConfigError::Validation(
                        "smtp_server cannot be empty when email_config is provided".to_string(),
                    ));
                }
                if email_config.credentials.username.is_empty() {
                    return Err(ConfigError::Validation(
                        "email username cannot be empty when email_config is provided".to_string(),
                    ));
                }
                if email_config.credentials.password.is_empty() {
                    tracing::warn!("Email password is empty - email functionality may not work");
                }
            }
            if self.port == 0 {
                return Err(ConfigError::Validation("port cannot be 0".to_string()));
            }
            match self.log_level.as_str() {
                "error" | "warn" | "info" | "debug" | "trace" => {}
                _ => {
                    return Err(ConfigError::Validation(format!(
                        "invalid log level '{}', must be one of: error, warn, info, debug, trace",
                        self.log_level,
                    )));
                }
            }
            tracing::info!("Configuration validation passed");
            Ok(())
        }

        /// Generate a new random encryption key and return it as base64
        pub fn generate_totp_key() -> String {
            use rand::RngCore;
            let mut key = [0u8; 32];
            rand::rng().fill_bytes(&mut key);
            BASE64_STANDARD.encode(key)
        }

        /// Generate a new random cookie secret and return it as base64
        pub fn generate_cookie_secret() -> String {
            use rand::RngCore;
            let mut secret = [0u8; 64];
            rand::rng().fill_bytes(&mut secret);
            BASE64_STANDARD.encode(secret)
        }
    }

    impl Default for Config {
        fn default() -> Self {
            let totp_key: [u8; 32] = rand::random();

            Self {
                cookies_secret: tower_cookies::Key::generate(),
                totp_encryption_key: aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_exact_iter(totp_key)
                    .expect("Valid AES256GCM key"),
                email_config: None,
                base_url: default_base_url(),
                port: default_port(),
                log_level: default_log_level(),
                family_name: None,
                family_description: None,
                public: false,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_key_generation() {
            let key = Config::generate_totp_key();
            let decoded = BASE64_STANDARD.decode(&key).unwrap();

            assert_eq!(decoded.len(), 32);

            let secret = Config::generate_cookie_secret();

            assert!(secret.len() >= 32);
        }

        #[test]
        fn test_validation() {
            let config = Config::default();

            assert!(config.validate().is_err());
        }
    }
}

pub mod client {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    pub struct Config {
        #[serde(default)]
        pub family_name: Option<String>,
        #[serde(default)]
        pub family_description: Option<String>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                family_name: None,
                family_description: None,
            }
        }
    }
}
