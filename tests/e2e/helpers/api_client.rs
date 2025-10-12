use anyhow::Result;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Method, Request, Response, StatusCode};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TestClient {
    base_url: String,
    client: Client<hyper_util::client::legacy::connect::HttpConnector, Full<Bytes>>,
}

impl TestClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder(TokioExecutor::new()).build_http();
        Self {
            base_url: base_url.to_string(),
            client,
        }
    }

    pub async fn get(&self, path: &str) -> Result<ApiResponse> {
        self.request::<()>(Method::GET, path, None, None).await
    }

    pub async fn get_with_auth(&self, path: &str, token: &str) -> Result<ApiResponse> {
        self.request::<()>(Method::GET, path, None, Some(token))
            .await
    }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<ApiResponse> {
        self.request(Method::POST, path, Some(body), None).await
    }

    pub async fn post_with_auth<T: Serialize>(
        &self,
        path: &str,
        body: &T,
        token: &str,
    ) -> Result<ApiResponse> {
        self.request(Method::POST, path, Some(body), Some(token))
            .await
    }

    #[allow(dead_code)]
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<ApiResponse> {
        self.request(Method::PUT, path, Some(body), None).await
    }

    pub async fn put_with_auth<T: Serialize>(
        &self,
        path: &str,
        body: &T,
        token: &str,
    ) -> Result<ApiResponse> {
        self.request(Method::PUT, path, Some(body), Some(token))
            .await
    }

    pub async fn patch<T: Serialize>(&self, path: &str, body: &T) -> Result<ApiResponse> {
        self.request(Method::PATCH, path, Some(body), None).await
    }

    pub async fn patch_with_auth<T: Serialize>(
        &self,
        path: &str,
        body: &T,
        token: &str,
    ) -> Result<ApiResponse> {
        self.request(Method::PATCH, path, Some(body), Some(token))
            .await
    }

    pub async fn delete(&self, path: &str) -> Result<ApiResponse> {
        self.request::<()>(Method::DELETE, path, None, None).await
    }

    pub async fn delete_with_auth(&self, path: &str, token: &str) -> Result<ApiResponse> {
        self.request::<()>(Method::DELETE, path, None, Some(token))
            .await
    }

    async fn request<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
        auth_token: Option<&str>,
    ) -> Result<ApiResponse> {
        let url = format!("{}{}", self.base_url, path);
        let mut req_builder = Request::builder().method(method).uri(&url);

        if let Some(token) = auth_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        let body_bytes = if let Some(body) = body {
            req_builder = req_builder.header("Content-Type", "application/json");
            Full::new(Bytes::from(serde_json::to_vec(body)?))
        } else {
            Full::new(Bytes::new())
        };

        let request = req_builder.body(body_bytes)?;
        let response = self.client.request(request).await?;

        ApiResponse::from_response(response).await
    }
}

pub struct ApiResponse {
    pub status: StatusCode,
    pub body: Option<Value>,
    pub body_bytes: Vec<u8>,
    pub headers: HashMap<String, String>,
}

impl ApiResponse {
    async fn from_response(response: Response<hyper::body::Incoming>) -> Result<Self> {
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();

        let body_bytes = response.into_body().collect().await?.to_bytes().to_vec();

        let body = if !body_bytes.is_empty() {
            serde_json::from_slice(&body_bytes).ok()
        } else {
            None
        };

        Ok(Self {
            status,
            body,
            body_bytes,
            headers,
        })
    }

    pub fn assert_status(&self, expected: StatusCode) -> &Self {
        assert_eq!(
            self.status, expected,
            "Expected status {} but got {}. Body: {:?}",
            expected, self.status, self.body
        );
        self
    }

    #[allow(dead_code)]
    pub fn assert_success(&self) -> &Self {
        assert!(
            self.status.is_success(),
            "Expected success status but got {}. Body: {:?}",
            self.status,
            self.body
        );
        self
    }

    /// Assert that the error response contains the expected message
    pub fn assert_error_message(&self, expected_message: &str) -> &Self {
        let message = self
            .body
            .as_ref()
            .and_then(|b| b.get("message"))
            .and_then(|m| m.as_str())
            .expect("Missing message field in error response");

        assert!(
            message.contains(expected_message),
            "Expected error message to contain '{}', but got '{}'",
            expected_message,
            message
        );
        self
    }

    #[allow(dead_code)]
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_slice(&self.body_bytes)?)
    }

    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    #[allow(dead_code)]
    pub fn assert_header(&self, name: &str, value: &str) -> &Self {
        let actual = self
            .headers
            .get(name)
            .unwrap_or_else(|| panic!("Header '{}' not found", name));
        assert_eq!(actual, value, "Header '{}' value mismatch", name);
        self
    }

    pub fn assert_header_exists(&self, name: &str) -> &Self {
        assert!(
            self.headers.contains_key(name),
            "Header '{}' not found",
            name
        );
        self
    }
}
