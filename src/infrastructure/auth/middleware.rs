use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::infrastructure::config::Config;
use crate::{
    domain::auth::JwtManager, error::AppError, infrastructure::repositories::UserRepository,
};
use uuid::Uuid;

/// User context injected into request extensions after authentication
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

/// Authentication middleware
pub async fn auth_middleware(
    State((user_repo, config)): State<(Arc<UserRepository>, Arc<Config>)>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    // Check Bearer token format
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized(
            "Invalid authorization format".to_string(),
        ));
    }

    let token = &auth_header[7..]; // Skip "Bearer "

    // Validate JWT token
    let jwt_manager = JwtManager::new(config.jwt_secret.clone(), config.jwt_expiration_hours);

    let claims = jwt_manager.validate_token(token)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

    // Verify user exists in database
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    // Add user context to request
    request.extensions_mut().insert(AuthUser {
        user_id: user.id,
        email: user.email,
    });

    Ok(next.run(request).await)
}
