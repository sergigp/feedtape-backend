use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

const GITHUB_AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_API_URL: &str = "https://api.github.com/user";
const GITHUB_USER_EMAIL_API_URL: &str = "https://api.github.com/user/emails";

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAccessToken {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

pub struct GitHubOAuthClient {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    http_client: reqwest::Client,
}

impl GitHubOAuthClient {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            http_client: reqwest::Client::new(),
        }
    }

    /// Generate the GitHub OAuth authorization URL
    pub fn get_authorization_url(&self, state: &str) -> String {
        format!(
            "{}?client_id={}&redirect_uri={}&scope=user:email&state={}",
            GITHUB_AUTHORIZE_URL, self.client_id, self.redirect_uri, state
        )
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str) -> AppResult<GitHubAccessToken> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];

        let response = self
            .http_client
            .post(GITHUB_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("GitHub token exchange failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Internal(format!(
                "GitHub token exchange failed: {}",
                error_text
            )));
        }

        response
            .json::<GitHubAccessToken>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse GitHub token: {}", e)))
    }

    /// Get user information from GitHub
    pub async fn get_user_info(&self, access_token: &str) -> AppResult<GitHubUser> {
        let mut user: GitHubUser = self
            .http_client
            .get(GITHUB_USER_API_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "FeedTape-Backend")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get GitHub user: {}", e)))?
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse GitHub user: {}", e)))?;

        // If email is not public, fetch from emails endpoint
        if user.email.is_none() {
            let emails: Vec<GitHubEmail> = self
                .http_client
                .get(GITHUB_USER_EMAIL_API_URL)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("User-Agent", "FeedTape-Backend")
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("Failed to get GitHub emails: {}", e)))?
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("Failed to parse GitHub emails: {}", e)))?;

            // Find primary verified email
            user.email = emails
                .iter()
                .find(|e| e.primary && e.verified)
                .or_else(|| emails.iter().find(|e| e.verified))
                .map(|e| e.email.clone());
        }

        Ok(user)
    }
}
