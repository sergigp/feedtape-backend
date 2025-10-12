use serde::Deserialize;
use std::env;

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
    // TTS Configuration
    pub tts_provider: TtsProvider,
    pub tts_cache_enabled: bool,
    // OpenAI TTS
    pub openai_api_key: Option<String>,
    pub openai_tts_model: String,
    pub openai_tts_voice: String,
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TtsProvider {
    OpenAI,
    Polly,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let config = Config {
            database_url: env::var("DATABASE_URL")?,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()?,
            refresh_token_expiration_days: env::var("REFRESH_TOKEN_EXPIRATION_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            aws_region: env::var("AWS_REGION").unwrap_or_else(|_| "eu-west-1".to_string()),
            environment: env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .parse::<String>()
                .map(|s| match s.as_str() {
                    "production" => Environment::Production,
                    _ => Environment::Development,
                })?,
            log_format: env::var("LOG_FORMAT")
                .unwrap_or_else(|_| "pretty".to_string())
                .parse::<String>()
                .map(|s| match s.as_str() {
                    "json" => LogFormat::Json,
                    _ => LogFormat::Pretty,
                })?,
            github_client_id: env::var("GITHUB_CLIENT_ID")?,
            github_client_secret: env::var("GITHUB_CLIENT_SECRET")?,
            github_redirect_uri: env::var("GITHUB_REDIRECT_URI")?,
            tts_provider: env::var("TTS_PROVIDER")
                .unwrap_or_else(|_| "openai".to_string())
                .parse::<String>()
                .map(|s| match s.to_lowercase().as_str() {
                    "polly" => TtsProvider::Polly,
                    _ => TtsProvider::OpenAI,
                })?,
            tts_cache_enabled: env::var("TTS_CACHE_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<String>()
                .map(|s| s.to_lowercase() == "true")
                .unwrap_or(false),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            openai_tts_model: env::var("OPENAI_TTS_MODEL")
                .unwrap_or_else(|_| "tts-1".to_string()),
            openai_tts_voice: env::var("OPENAI_TTS_VOICE")
                .unwrap_or_else(|_| "alloy".to_string()),
        };

        Ok(config)
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }
}
