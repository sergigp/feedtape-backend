use serde::Deserialize;
use std::env;
use std::fmt;

#[derive(Debug)]
pub struct ConfigError {
    var_name: String,
    message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration error: {} - {}", self.var_name, self.message)
    }
}

impl std::error::Error for ConfigError {}

fn required_env(name: &str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError {
        var_name: name.to_string(),
        message: "environment variable is required but not set".to_string(),
    })
}

fn parse_env<T: std::str::FromStr>(name: &str, value: String) -> Result<T, ConfigError> {
    value.parse().map_err(|_| ConfigError {
        var_name: name.to_string(),
        message: format!("failed to parse value '{}'", value),
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub refresh_token_expiration_days: i64,
    pub aws_region: String,
    pub environment: Environment,
    pub log_format: LogFormat,
    // GitHub OAuth
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    // TTS Cache
    pub tts_cache_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Pretty,
    Json,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let port_str = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let jwt_exp_str = env::var("JWT_EXPIRATION_HOURS").unwrap_or_else(|_| "1".to_string());
        let refresh_exp_str =
            env::var("REFRESH_TOKEN_EXPIRATION_DAYS").unwrap_or_else(|_| "30".to_string());

        let config = Config {
            database_url: required_env("DATABASE_URL")?,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: parse_env("PORT", port_str)?,
            jwt_secret: required_env("JWT_SECRET")?,
            jwt_expiration_hours: parse_env("JWT_EXPIRATION_HOURS", jwt_exp_str)?,
            refresh_token_expiration_days: parse_env(
                "REFRESH_TOKEN_EXPIRATION_DAYS",
                refresh_exp_str,
            )?,
            aws_region: env::var("AWS_REGION").unwrap_or_else(|_| "eu-west-1".to_string()),
            environment: match env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .as_str()
            {
                "production" => Environment::Production,
                _ => Environment::Development,
            },
            log_format: match env::var("LOG_FORMAT")
                .unwrap_or_else(|_| "pretty".to_string())
                .as_str()
            {
                "json" => LogFormat::Json,
                _ => LogFormat::Pretty,
            },
            github_client_id: required_env("GITHUB_CLIENT_ID")?,
            github_client_secret: required_env("GITHUB_CLIENT_SECRET")?,
            github_redirect_uri: required_env("GITHUB_REDIRECT_URI")?,
            tts_cache_enabled: env::var("TTS_CACHE_ENABLED")
                .map(|s| s.to_lowercase() == "true")
                .unwrap_or(false),
        };

        Ok(config)
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }
}
