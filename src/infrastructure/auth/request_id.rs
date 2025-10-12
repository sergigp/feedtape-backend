use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

pub const X_REQUEST_ID: &str = "x-request-id";

/// Middleware to generate and attach request ID to each request
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Generate a unique request ID
    let request_id = Uuid::new_v4().to_string();

    // Add request ID to request extensions for use in handlers
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Process the request
    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(X_REQUEST_ID, header_value);
    }

    response
}

/// Request ID wrapper type for extension
#[derive(Debug, Clone)]
pub struct RequestId(pub String);
