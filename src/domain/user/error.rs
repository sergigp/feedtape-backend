use crate::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("dependency error: {0}")]
    Dependency(String),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<AppError> for UserServiceError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::BadRequest(msg) => UserServiceError::Invalid(msg),
            AppError::NotFound(_) => UserServiceError::NotFound,
            _ => UserServiceError::Dependency(err.to_string()),
        }
    }
}

impl From<UserServiceError> for AppError {
    fn from(err: UserServiceError) -> Self {
        match err {
            UserServiceError::Invalid(msg) => AppError::BadRequest(msg),
            UserServiceError::NotFound => AppError::NotFound("User not found".to_string()),
            UserServiceError::Dependency(msg) => AppError::Internal(msg),
            UserServiceError::Other(e) => AppError::Internal(e.to_string()),
        }
    }
}
