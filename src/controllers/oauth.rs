use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::auth::{AuthService, TokenResponse},
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

/// Generate HTML page that redirects to mobile app deep link
fn generate_mobile_redirect_html(tokens: &TokenResponse) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <title>Authentication Successful</title>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    body {{
      font-family: system-ui, -apple-system, sans-serif;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      margin: 0;
      background: #f5f5f5;
    }}
    .container {{
      text-align: center;
      padding: 2rem;
      background: white;
      border-radius: 12px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    }}
    .success {{
      font-size: 3rem;
      margin-bottom: 1rem;
    }}
    h1 {{
      color: #333;
      margin-bottom: 0.5rem;
    }}
    p {{
      color: #666;
    }}
    .spinner {{
      border: 3px solid #f3f3f3;
      border-top: 3px solid #333;
      border-radius: 50%;
      width: 40px;
      height: 40px;
      animation: spin 1s linear infinite;
      margin: 1rem auto;
    }}
    @keyframes spin {{
      0% {{ transform: rotate(0deg); }}
      100% {{ transform: rotate(360deg); }}
    }}
  </style>
</head>
<body>
  <div class="container">
    <div class="success">✓</div>
    <h1>Authentication Successful!</h1>
    <p>Redirecting you back to the app...</p>
    <div class="spinner"></div>
  </div>

  <script>
    // Try to redirect to the app
    const deepLink = 'feedtape://auth/callback?' +
      'token=' + encodeURIComponent('{token}') +
      '&refresh_token=' + encodeURIComponent('{refresh_token}') +
      '&expires_in={expires_in}';

    // Attempt redirect
    window.location.href = deepLink;

    // Fallback: show manual close instruction after 2 seconds
    setTimeout(() => {{
      const container = document.querySelector('.container');
      container.innerHTML = `
        <div class="success">✓</div>
        <h1>Authentication Complete</h1>
        <p>You can now close this window and return to the app.</p>
      `;
    }}, 2000);
  </script>
</body>
</html>"#,
        token = tokens.token,
        refresh_token = tokens.refresh_token,
        expires_in = tokens.expires_in
    )
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
    /// - HTML page with deep link redirect (for mobile clients)
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
            let html = generate_mobile_redirect_html(&tokens);
            Ok(Html(html).into_response())
        } else {
            Ok(Json(tokens).into_response())
        }
    }
}
