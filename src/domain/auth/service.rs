use crate::infrastructure::repositories::{RefreshTokenRepository, UserRepository};
use super::{generate_refresh_token, JwtManager, TokenResponse};
use super::error::AuthServiceError;
use crate::domain::user::User;
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

pub struct AuthService {
    user_repo: Arc<UserRepository>,
    refresh_token_repo: Arc<RefreshTokenRepository>,
    jwt_secret: String,
    jwt_expiration_hours: i64,
    refresh_token_expiration_days: i64,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        refresh_token_repo: Arc<RefreshTokenRepository>,
        jwt_secret: String,
        jwt_expiration_hours: i64,
        refresh_token_expiration_days: i64,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            jwt_secret,
            jwt_expiration_hours,
            refresh_token_expiration_days,
        }
    }
}

#[async_trait]
pub trait AuthServiceApi: Send + Sync {
    async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, AuthServiceError>;

    async fn logout(&self, refresh_token: &str) -> Result<(), AuthServiceError>;

    async fn logout_all(&self, user_id: Uuid) -> Result<(), AuthServiceError>;

    async fn create_tokens_for_user(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> Result<TokenResponse, AuthServiceError>;
}

#[async_trait]
impl AuthServiceApi for AuthService {
    async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, AuthServiceError> {
        let (user_id, _expires_at) = self.find_valid_refresh_token(refresh_token).await?;
        let user = self.find_user(user_id).await?;
        let access_token = self.generate_access_token(user.id, &user.email)?;
        let new_refresh_token = generate_refresh_token();

        self.refresh_token_repo.revoke(refresh_token).await
            .map_err(|e| AuthServiceError::Dependency(e.to_string()))?;

        self.store_refresh_token(user.id, &new_refresh_token).await?;

        Ok(TokenResponse {
            token: access_token,
            refresh_token: new_refresh_token,
            expires_in: self.jwt_expiration_hours * 3600,
        })
    }

    async fn logout(&self, refresh_token: &str) -> Result<(), AuthServiceError> {
        self.refresh_token_repo.revoke(refresh_token).await
            .map_err(|e| AuthServiceError::Dependency(e.to_string()))
    }

    async fn logout_all(&self, user_id: Uuid) -> Result<(), AuthServiceError> {
        self.refresh_token_repo.revoke_all_for_user(user_id).await
            .map_err(|e| AuthServiceError::Dependency(e.to_string()))
    }

    async fn create_tokens_for_user(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> Result<TokenResponse, AuthServiceError> {
        let access_token = self.generate_access_token(user_id, email)?;
        let refresh_token = generate_refresh_token();

        self.store_refresh_token(user_id, &refresh_token).await?;

        Ok(TokenResponse {
            token: access_token,
            refresh_token,
            expires_in: self.jwt_expiration_hours * 3600,
        })
    }
}

impl AuthService {
    async fn find_valid_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<(Uuid, DateTime<Utc>), AuthServiceError> {
        if let Some((revoked, is_expired)) = self.refresh_token_repo
            .check_token_status(refresh_token)
            .await
            .map_err(|e| AuthServiceError::Dependency(e.to_string()))?
        {
            if revoked || is_expired {
                return Err(AuthServiceError::Expired);
            }
        } else {
            return Err(AuthServiceError::Invalid("Token not found".to_string()));
        }

        self.refresh_token_repo
            .find_valid(refresh_token)
            .await
            .map_err(|e| AuthServiceError::Dependency(e.to_string()))?
            .ok_or_else(|| AuthServiceError::Invalid("Token not valid".to_string()))
    }

    async fn find_user(&self, user_id: Uuid) -> Result<User, AuthServiceError> {
        self.user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| AuthServiceError::Dependency(e.to_string()))?
            .ok_or_else(|| AuthServiceError::Unauthorized("User not found".to_string()))
    }

    fn generate_access_token(&self, user_id: Uuid, email: &str) -> Result<String, AuthServiceError> {
        let jwt_manager = JwtManager::new(
            self.jwt_secret.clone(),
            self.jwt_expiration_hours,
        );
        jwt_manager
            .generate_token(user_id, email)
            .map_err(|e| AuthServiceError::from(e))
    }

    async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token: &str,
    ) -> Result<(), AuthServiceError> {
        self.refresh_token_repo
            .create(user_id, token, self.refresh_token_expiration_days)
            .await
            .map_err(|e| AuthServiceError::Dependency(e.to_string()))
    }
}
