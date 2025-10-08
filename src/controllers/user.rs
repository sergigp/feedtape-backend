use axum::{extract::State, http::StatusCode, Extension, Json};
use std::sync::Arc;

use crate::{
    error::AppResult,
    infrastructure::auth::AuthUser,
    domain::user::UserService,
};
use crate::domain::user::{MeResponse, UpdateMeRequest};

pub struct UserController {
    user_service: Arc<UserService>,
}

impl UserController {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }

    /// GET /api/me - Get current user profile
    pub async fn get_me(
        State(controller): State<Arc<UserController>>,
        Extension(auth_user): Extension<AuthUser>,
    ) -> AppResult<Json<MeResponse>> {
        let response = controller.user_service.get_user_profile(auth_user.user_id).await?;
        Ok(Json(response))
    }

    /// PATCH /api/me - Update user settings
    pub async fn update_me(
        State(controller): State<Arc<UserController>>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<UpdateMeRequest>,
    ) -> AppResult<StatusCode> {
        let settings = request
            .settings
            .ok_or_else(|| crate::error::AppError::BadRequest("Settings are required".to_string()))?;

        controller.user_service.update_user_settings(auth_user.user_id, settings).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
