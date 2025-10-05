use axum::{extract::State, http::StatusCode, Extension, Json};
use std::sync::Arc;

use crate::{
    error::AppResult,
    infrastructure::auth::AuthUser,
    domain::auth::AuthService,
};
use crate::domain::auth::{RefreshTokenRequest, TokenResponse};

pub struct AuthController {
    auth_service: Arc<AuthService>,
}

impl AuthController {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    /// POST /auth/refresh - Refresh access token
    pub async fn refresh(
        State(controller): State<Arc<AuthController>>,
        Json(request): Json<RefreshTokenRequest>,
    ) -> AppResult<Json<TokenResponse>> {
        let response = controller.auth_service.refresh_token(&request.refresh_token).await?;
        Ok(Json(response))
    }

    /// POST /auth/logout - Logout (revoke refresh token)
    pub async fn logout(
        State(controller): State<Arc<AuthController>>,
        Json(request): Json<RefreshTokenRequest>,
    ) -> AppResult<StatusCode> {
        controller.auth_service.logout(&request.refresh_token).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    /// POST /auth/logout/all - Logout from all devices
    pub async fn logout_all(
        State(controller): State<Arc<AuthController>>,
        Extension(auth_user): Extension<AuthUser>,
    ) -> AppResult<StatusCode> {
        controller.auth_service.logout_all(auth_user.user_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
