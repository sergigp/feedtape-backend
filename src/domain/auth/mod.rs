pub mod error;
pub mod jwt;
pub mod service;

pub use error::AuthServiceError;
pub use jwt::{generate_refresh_token, Claims, JwtManager};
use serde::{Deserialize, Serialize};
pub use service::{AuthService, AuthServiceApi};

/// Token response for OAuth callbacks
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Refresh token request
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}
