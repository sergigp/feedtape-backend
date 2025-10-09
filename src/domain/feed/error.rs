use crate::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum FeedServiceError {
    #[error("dependency error: {0}")]
    Dependency(String),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("feed not found")]
    NotFound,
    #[error("feed already exists")]
    Conflict,
    #[error("payment required: {0}")]
    PaymentRequired(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<AppError> for FeedServiceError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::BadRequest(msg) => FeedServiceError::Invalid(msg),
            AppError::NotFound(_) => FeedServiceError::NotFound,
            AppError::Conflict(_) => FeedServiceError::Conflict,
            AppError::PaymentRequired(msg) => FeedServiceError::PaymentRequired(msg),
            _ => FeedServiceError::Dependency(err.to_string()),
        }
    }
}

impl From<FeedServiceError> for AppError {
    fn from(err: FeedServiceError) -> Self {
        match err {
            FeedServiceError::Invalid(msg) => AppError::BadRequest(msg),
            FeedServiceError::NotFound => AppError::NotFound("Feed not found".to_string()),
            FeedServiceError::Conflict => AppError::Conflict("Feed URL already exists".to_string()),
            FeedServiceError::PaymentRequired(msg) => AppError::PaymentRequired(msg),
            FeedServiceError::Dependency(msg) => AppError::Internal(msg),
            FeedServiceError::Other(e) => AppError::Internal(e.to_string()),
        }
    }
}
