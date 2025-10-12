use crate::e2e::helpers;

use helpers::TestContext;
use hyper::StatusCode;
use test_context::test_context;

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_return_ok_for_health_check(ctx: &TestContext) {

    let response = ctx.client.get("/health").await.unwrap();

    response.assert_status(StatusCode::OK);

    // Health endpoint returns plain text
    let body = String::from_utf8(response.body_bytes.clone()).unwrap();
    assert_eq!(body, "OK");
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_return_ready_status(ctx: &TestContext) {

    let response = ctx.client.get("/health/ready").await.unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();

    // Check readiness response structure
    assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("ready"));
    assert!(body.get("database").is_some());
    assert!(body.get("tts").is_some());
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_not_require_auth_for_health_checks(ctx: &TestContext) {

    // Both health endpoints should work without authentication
    let response = ctx.client.get("/health").await.unwrap();
    response.assert_status(StatusCode::OK);

    let response = ctx.client.get("/health/ready").await.unwrap();
    response.assert_status(StatusCode::OK);
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_include_request_id_in_health_responses(ctx: &TestContext) {

    let response = ctx.client.get("/health").await.unwrap();
    response.assert_header_exists("x-request-id");

    let response = ctx.client.get("/health/ready").await.unwrap();
    response.assert_header_exists("x-request-id");
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_handle_database_connectivity_check(ctx: &TestContext) {

    // The ready endpoint should verify database connection
    let response = ctx.client.get("/health/ready").await.unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let db_status = body.get("database").and_then(|v| v.as_str());

    // Database should be connected since we're using testcontainers
    assert_eq!(db_status, Some("connected"));
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_be_fast_health_check(ctx: &TestContext) {

    let start = std::time::Instant::now();
    let response = ctx.client.get("/health").await.unwrap();
    let duration = start.elapsed();

    response.assert_status(StatusCode::OK);

    // Health check should be very fast (under 100ms)
    assert!(
        duration.as_millis() < 100,
        "Health check took too long: {:?}",
        duration
    );
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_handle_concurrent_health_checks(ctx: &TestContext) {

    // Simulate multiple concurrent health checks
    let mut futures = Vec::new();
    for _ in 0..10 {
        let client = ctx.client.clone();
        futures.push(async move { client.get("/health").await });
    }

    let results = futures::future::join_all(futures).await;

    // All health checks should succeed
    for result in results {
        let response = result.unwrap();
        response.assert_status(StatusCode::OK);
    }
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_return_service_details_in_ready(ctx: &TestContext) {

    let response = ctx.client.get("/health/ready").await.unwrap();
    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();

    // Verify all expected services are reported
    let expected_services = vec!["database", "tts"];

    for service in expected_services {
        assert!(
            body.get(service).is_some(),
            "Missing service '{}' in ready response",
            service
        );

        let status = body.get(service).and_then(|v| v.as_str());
        assert!(status.is_some(), "Service '{}' has no status", service);
    }
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_use_different_endpoints_for_liveness_and_readiness(ctx: &TestContext) {

    // /health is for liveness (is the service running?)
    let liveness_response = ctx.client.get("/health").await.unwrap();
    liveness_response.assert_status(StatusCode::OK);

    // /health/ready is for readiness (is the service ready to handle requests?)
    let readiness_response = ctx.client.get("/health/ready").await.unwrap();
    readiness_response.assert_status(StatusCode::OK);

    // They should return different response types
    assert!(liveness_response.body.is_none()); // Plain text
    assert!(readiness_response.body.is_some()); // JSON
}
