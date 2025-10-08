use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// Main application error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Refresh token expired")]
    RefreshTokenExpired,

    #[error("Invalid input: {0}")]
    BadRequest(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Payment required: {0}")]
    PaymentRequired(String),

    #[error("Text too large: {0}")]
    PayloadTooLarge(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Error response structure - simplified to just message + status code
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized(_) | Self::InvalidRefreshToken | Self::RefreshTokenExpired => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::PaymentRequired(_) => StatusCode::PAYMENT_REQUIRED,
            Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Database(_) | Self::ExternalService(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Convert to simplified error response
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            message: self.to_string(),
        }
    }
}

/// Implement IntoResponse for automatic conversion in handlers
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the error
        let status = self.status_code();
        tracing::error!(
            error = %self,
            status = %status.as_u16(),
            "Request failed"
        );

        // Create simplified error response
        let error_response = self.to_response();

        (status, Json(error_response)).into_response()
    }
}

/// Custom result type for the application
pub type AppResult<T> = Result<T, AppError>;
