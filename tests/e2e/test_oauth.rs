use crate::e2e::helpers;

use helpers::TestContext;
use hyper::StatusCode;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn it_should_initiate_github_oauth() {
    let ctx = TestContext::new().await.unwrap();

    let response = ctx.client.get("/auth/oauth/github").await.unwrap();

    // Should redirect to GitHub
    assert!(
        response.status == StatusCode::TEMPORARY_REDIRECT
            || response.status == StatusCode::MOVED_PERMANENTLY
            || response.status == StatusCode::FOUND,
        "Expected redirect, got: {:?}",
        response.status
    );

    // Should have Location header pointing to GitHub
    let location = response.header("location");
    assert!(location.is_some(), "Missing Location header");
    let location = location.unwrap();
    assert!(
        location.starts_with("https://github.com/login/oauth/authorize"),
        "Location should point to GitHub OAuth, got: {}",
        location
    );
    assert!(location.contains("client_id="), "Should include client_id");
    assert!(
        location.contains("state="),
        "Should include state parameter"
    );

    // State should be prefixed with "web:" (either raw or URL-encoded)
    assert!(
        location.contains("state=web:") || location.contains("state=web%3A"),
        "State should be prefixed with 'web:', got: {}",
        location
    );
}

#[tokio::test]
#[serial]
async fn it_should_initiate_github_oauth_with_mobile_param() {
    let ctx = TestContext::new().await.unwrap();

    let response = ctx
        .client
        .get("/auth/oauth/github?mobile=true")
        .await
        .unwrap();

    // Should redirect to GitHub
    assert!(
        response.status == StatusCode::TEMPORARY_REDIRECT
            || response.status == StatusCode::MOVED_PERMANENTLY
            || response.status == StatusCode::FOUND,
        "Expected redirect, got: {:?}",
        response.status
    );

    // Should have Location header pointing to GitHub
    let location = response.header("location");
    assert!(location.is_some(), "Missing Location header");
    let location = location.unwrap();
    assert!(
        location.starts_with("https://github.com/login/oauth/authorize"),
        "Location should point to GitHub OAuth, got: {}",
        location
    );

    // State should be prefixed with "mobile:" (either raw or URL-encoded)
    assert!(
        location.contains("state=mobile:") || location.contains("state=mobile%3A"),
        "State should be prefixed with 'mobile:' for mobile requests, got: {}",
        location
    );
}

#[tokio::test]
#[serial]
async fn it_should_reject_callback_without_code() {
    let ctx = TestContext::new().await.unwrap();

    // Try to access callback without code parameter
    let response = ctx.client.get("/auth/callback/github").await.unwrap();

    // Should return an error (400 or 422 for missing required query param)
    assert!(
        response.status == StatusCode::BAD_REQUEST
            || response.status == StatusCode::UNPROCESSABLE_ENTITY
            || response.status == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected error for missing code, got: {:?}",
        response.status
    );
}

#[tokio::test]
#[serial]
async fn it_should_reject_callback_without_state() {
    let ctx = TestContext::new().await.unwrap();

    // Try to access callback with code but without state parameter
    let response = ctx
        .client
        .get("/auth/callback/github?code=test123")
        .await
        .unwrap();

    // Should return an error (400 or 422 for missing required query param)
    assert!(
        response.status == StatusCode::BAD_REQUEST
            || response.status == StatusCode::UNPROCESSABLE_ENTITY
            || response.status == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected error for missing state, got: {:?}",
        response.status
    );
}

// Note: Full OAuth flow testing would require mocking GitHub's OAuth endpoints
// which is complex. For now, we test the basic endpoint existence and redirect behavior.
// In production, you'd want to:
// 1. Mock the GitHub OAuth server
// 2. Test the full flow: initiate -> callback with valid code -> get tokens
// 3. Test error cases: invalid code, missing email, etc.
