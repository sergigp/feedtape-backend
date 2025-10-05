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

/// Error response structure matching OpenAPI spec
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    pub request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_url: Option<String>,
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

    /// Get the error code string
    pub fn error_code(&self) -> String {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::InvalidRefreshToken => "INVALID_REFRESH_TOKEN",
            Self::RefreshTokenExpired => "REFRESH_TOKEN_EXPIRED",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            Self::PaymentRequired(_) => "UPGRADE_REQUIRED",
            Self::PayloadTooLarge(_) => "PAYLOAD_TOO_LARGE",
            Self::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
        .to_string()
    }

    /// Get help URL for this error type
    pub fn help_url(&self) -> Option<String> {
        match self {
            Self::PaymentRequired(_) => Some("https://feedtape.app/upgrade".to_string()),
            Self::RateLimitExceeded(_) => Some("https://feedtape.app/docs/limits".to_string()),
            _ => None,
        }
    }

    /// Convert to error response with request ID
    pub fn to_response(&self, request_id: String) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.error_code(),
                message: self.to_string(),
                details: None,
                help_url: self.help_url(),
            },
            request_id,
        }
    }
}

/// Implement IntoResponse for automatic conversion in handlers
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Generate request ID
        let request_id = uuid::Uuid::new_v4().to_string();

        // Log the error
        let status = self.status_code();
        tracing::error!(
            error = %self,
            request_id = %request_id,
            status = %status.as_u16(),
            "Request failed"
        );

        // Create error response
        let error_response = self.to_response(request_id);

        (status, Json(error_response)).into_response()
    }
}

/// Custom result type for the application
pub type AppResult<T> = Result<T, AppError>;
