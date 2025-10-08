use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::auth::AuthService,
    error::AppResult,
    infrastructure::{oauth::GitHubOAuthClient, repositories::UserRepository},
};

#[derive(Debug, Deserialize)]
pub struct InitiateOAuthParams {
    pub mobile: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
}

pub struct OAuthController {
    github_client: Arc<GitHubOAuthClient>,
    user_repo: Arc<UserRepository>,
    auth_service: Arc<AuthService>,
}

impl OAuthController {
    pub fn new(
        github_client: Arc<GitHubOAuthClient>,
        user_repo: Arc<UserRepository>,
        auth_service: Arc<AuthService>,
    ) -> Self {
        Self {
            github_client,
            user_repo,
            auth_service,
        }
    }

    /// GET /auth/oauth/github - Initiate GitHub OAuth flow
    ///
    /// Query params:
    /// - mobile: Optional boolean. If true, callback will redirect to mobile deep link
    pub async fn initiate_github(
        State(controller): State<Arc<OAuthController>>,
        Query(params): Query<InitiateOAuthParams>,
    ) -> impl IntoResponse {
        // Generate random UUID for CSRF protection
        let uuid = Uuid::new_v4().to_string();

        // Encode mobile indicator in state: "mobile:UUID" or "web:UUID"
        let state = if params.mobile.unwrap_or(false) {
            format!("mobile:{}", uuid)
        } else {
            format!("web:{}", uuid)
        };

        // TODO: Store state in session/cache for validation (currently simplified)
        // In production, you'd store this with expiry in Redis or DB

        let auth_url = controller.github_client.get_authorization_url(&state);

        Redirect::temporary(&auth_url)
    }

    /// GET /auth/callback/github - Handle GitHub OAuth callback
    ///
    /// Returns either:
    /// - JSON with tokens (for web clients)
    /// - Redirect to deep link (for mobile clients)
    pub async fn github_callback(
        State(controller): State<Arc<OAuthController>>,
        Query(params): Query<OAuthCallbackParams>,
    ) -> AppResult<Response> {
        // Parse state to detect if this is a mobile request
        let is_mobile = params.state.starts_with("mobile:");

        // TODO: Validate state parameter against stored value
        // For now, we skip this check for simplicity

        // Exchange code for access token
        let token_response = controller.github_client.exchange_code(&params.code).await?;

        // Get user info from GitHub
        let github_user = controller
            .github_client
            .get_user_info(&token_response.access_token)
            .await?;

        // Validate we have an email
        let email = github_user.email.ok_or_else(|| {
            crate::error::AppError::BadRequest(
                "GitHub account has no verified email address".to_string(),
            )
        })?;

        let provider_id = github_user.id.to_string();

        // Check if user already exists
        let user = match controller
            .user_repo
            .find_by_oauth("github", &provider_id)
            .await?
        {
            Some(existing_user) => existing_user,
            None => {
                // Create new user
                controller
                    .user_repo
                    .create(&email, "github", &provider_id)
                    .await?
            }
        };

        // Generate JWT and refresh tokens
        let tokens = controller
            .auth_service
            .create_tokens_for_user(user.id, &user.email)
            .await?;

        // Return appropriate response based on client type
        if is_mobile {
            // Build deep link URL with tokens
            let deep_link = format!(
                "feedtape://auth/callback?token={}&refresh_token={}&expires_in={}",
                urlencoding::encode(&tokens.token),
                urlencoding::encode(&tokens.refresh_token),
                tokens.expires_in
            );
            Ok(Redirect::temporary(&deep_link).into_response())
        } else {
            Ok(Json(tokens).into_response())
        }
    }
}
