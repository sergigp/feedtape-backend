use crate::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum TtsServiceError {
    #[error("dependency error: {0}")]
    Dependency(String),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("payment required: {0}")]
    PaymentRequired(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<AppError> for TtsServiceError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::PaymentRequired(msg) => TtsServiceError::PaymentRequired(msg),
            AppError::BadRequest(msg) => TtsServiceError::Invalid(msg),
            _ => TtsServiceError::Dependency(err.to_string()),
        }
    }
}

impl From<TtsServiceError> for AppError {
    fn from(err: TtsServiceError) -> Self {
        match err {
            TtsServiceError::PaymentRequired(msg) => AppError::PaymentRequired(msg),
            TtsServiceError::Invalid(msg) => AppError::BadRequest(msg),
            TtsServiceError::Dependency(msg) => AppError::ExternalService(msg),
            TtsServiceError::Other(e) => AppError::Internal(e.to_string()),
        }
    }
}
