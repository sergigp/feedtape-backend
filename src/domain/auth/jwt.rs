use crate::error::{AppError, AppResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub email: String,
    pub exp: i64,     // Expiration time
    pub iat: i64,     // Issued at
}

pub struct JwtManager {
    secret: String,
    expiration_hours: i64,
}

impl JwtManager {
    pub fn new(secret: String, expiration_hours: i64) -> Self {
        Self {
            secret,
            expiration_hours,
        }
    }

    /// Generate a JWT access token for a user
    pub fn generate_token(&self, user_id: Uuid, email: &str) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiration_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
    }

    /// Validate a JWT token and extract claims
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
    }

    /// Extract user ID from token
    pub fn extract_user_id(&self, token: &str) -> AppResult<Uuid> {
        let claims = self.validate_token(token)?;
        Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))
    }
}

/// Generate a random refresh token
pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}
