use crate::{
    error::{AppError, AppResult},
    infrastructure::repositories::{RefreshTokenRepository, UserRepository},
};
use super::{generate_refresh_token, JwtManager, TokenResponse};
use uuid::Uuid;
use crate::infrastructure::config::Config;
use std::sync::Arc;

pub struct AuthService {
    user_repo: Arc<UserRepository>,
    refresh_token_repo: Arc<RefreshTokenRepository>,
    config: Arc<Config>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        refresh_token_repo: Arc<RefreshTokenRepository>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            config,
        }
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> AppResult<TokenResponse> {
        // First check if token exists and its status
        if let Some((revoked, is_expired)) = self.refresh_token_repo.check_token_status(refresh_token).await? {
            if revoked || is_expired {
                return Err(AppError::RefreshTokenExpired);
            }
        } else {
            return Err(AppError::InvalidRefreshToken);
        }

        // Find and validate refresh token (this should now always succeed based on above check)
        let (user_id, _expires_at) = self.refresh_token_repo.find_valid(refresh_token)
            .await?
            .ok_or_else(|| AppError::InvalidRefreshToken)?;

        // Get user
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

        // Generate new access token
        let jwt_manager = JwtManager::new(
            self.config.jwt_secret.clone(),
            self.config.jwt_expiration_hours,
        );
        let access_token = jwt_manager.generate_token(user.id, &user.email)?;

        // Generate new refresh token
        let new_refresh_token = generate_refresh_token();

        // Revoke old refresh token
        self.refresh_token_repo.revoke(refresh_token).await?;

        // Store new refresh token
        self.refresh_token_repo.create(
            user.id,
            &new_refresh_token,
            self.config.refresh_token_expiration_days,
        )
        .await?;

        Ok(TokenResponse {
            token: access_token,
            refresh_token: new_refresh_token,
            expires_in: self.config.jwt_expiration_hours * 3600,
        })
    }

    /// Logout user by revoking refresh token
    pub async fn logout(&self, refresh_token: &str) -> AppResult<()> {
        self.refresh_token_repo.revoke(refresh_token).await?;
        Ok(())
    }

    /// Logout user from all devices by revoking all refresh tokens
    pub async fn logout_all(&self, user_id: Uuid) -> AppResult<()> {
        self.refresh_token_repo.revoke_all_for_user(user_id).await?;
        Ok(())
    }

    /// Create tokens for a user (used after OAuth authentication)
    pub async fn create_tokens_for_user(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> AppResult<TokenResponse> {
        // Generate access token
        let jwt_manager = JwtManager::new(
            self.config.jwt_secret.clone(),
            self.config.jwt_expiration_hours,
        );
        let access_token = jwt_manager.generate_token(user_id, email)?;

        // Generate refresh token
        let refresh_token = generate_refresh_token();

        // Store refresh token
        self.refresh_token_repo.create(
            user_id,
            &refresh_token,
            self.config.refresh_token_expiration_days,
        )
        .await?;

        Ok(TokenResponse {
            token: access_token,
            refresh_token,
            expires_in: self.config.jwt_expiration_hours * 3600,
        })
    }
}
