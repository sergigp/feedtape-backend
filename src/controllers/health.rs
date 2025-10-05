use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;
use crate::infrastructure::db::{check_connection, DbPool};

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn health_ready(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    match check_connection(&pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "ready",
                "database": "connected",
                "tts": "available"
            })),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "not_ready",
                "database": "disconnected",
                "tts": "unknown"
            })),
        ),
    }
}
