use crate::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum AuthServiceError {
    #[error("dependency error: {0}")]
    Dependency(String),
    #[error("invalid token: {0}")]
    Invalid(String),
    #[error("token expired")]
    Expired,
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<AppError> for AuthServiceError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::InvalidRefreshToken => {
                AuthServiceError::Invalid("Invalid refresh token".to_string())
            }
            AppError::RefreshTokenExpired => AuthServiceError::Expired,
            AppError::Unauthorized(msg) => AuthServiceError::Unauthorized(msg),
            _ => AuthServiceError::Dependency(err.to_string()),
        }
    }
}

impl From<AuthServiceError> for AppError {
    fn from(err: AuthServiceError) -> Self {
        match err {
            AuthServiceError::Invalid(_) => AppError::InvalidRefreshToken,
            AuthServiceError::Expired => AppError::RefreshTokenExpired,
            AuthServiceError::Unauthorized(msg) => AppError::Unauthorized(msg),
            AuthServiceError::Dependency(msg) => AppError::Internal(msg),
            AuthServiceError::Other(e) => AppError::Internal(e.to_string()),
        }
    }
}
