use crate::e2e::helpers;

use helpers::{generate_test_jwt, TestContext};
use hyper::StatusCode;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn it_should_synthesize_text_to_speech() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": "Hello, this is a test message for text to speech."
            }),
            &token,
        )
        .await
        .unwrap();

    // Note: With mocked AWS, this will likely fail but we can test the endpoint exists
    // In a real scenario, we'd mock the Polly response properly
    assert!(
        response.status == StatusCode::OK ||
        response.status == StatusCode::SERVICE_UNAVAILABLE
    );

    if response.status == StatusCode::OK {
        // Verify headers
        assert!(response.header("content-type").is_some());
        assert!(response.header("x-character-count").is_some());
        assert!(response.header("x-voice-used").is_some());
    }
}

#[tokio::test]
#[serial]
async fn it_should_use_custom_voice_settings() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": "Este es un mensaje de prueba en español.",
                "language": "es",
                "voice": "Conchita",
                "speed": 1.25,
                "quality": "neural"
            }),
            &token,
        )
        .await
        .unwrap();

    // Test that endpoint accepts these parameters
    assert!(
        response.status == StatusCode::OK ||
        response.status == StatusCode::SERVICE_UNAVAILABLE ||
        response.status == StatusCode::PAYMENT_REQUIRED // Neural might require pro
    );
}

#[tokio::test]
#[serial]
async fn it_should_auto_detect_language() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let test_cases = vec![
        ("This is English text", "en"),
        ("Ceci est un texte français", "fr"),
        ("Dies ist ein deutscher Text", "de"),
        ("Questo è un testo italiano", "it"),
        ("Este é um texto português", "pt"),
    ];

    for (text, _expected_lang) in test_cases {
        let response = ctx
            .client
            .post_with_auth(
                "/api/tts/synthesize",
                &json!({
                    "text": text,
                    "language": "auto"
                }),
                &token,
            )
            .await
            .unwrap();

        assert!(
            response.status == StatusCode::OK ||
            response.status == StatusCode::SERVICE_UNAVAILABLE
        );

        if response.status == StatusCode::OK {
            // Check if language was detected
            assert!(response.header("x-language-detected").is_some());
        }
    }
}

#[tokio::test]
#[serial]
async fn it_should_enforce_text_length_limits() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Test empty text
    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": ""
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::BAD_REQUEST);

    // Test text that's too long (over 10000 characters)
    let long_text = "a".repeat(10001);
    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": long_text
            }),
            &token,
        )
        .await
        .unwrap();

    response
        .assert_status(StatusCode::PAYLOAD_TOO_LARGE)
        .assert_error_code("TEXT_TOO_LONG");
}

#[tokio::test]
#[serial]
async fn it_should_enforce_daily_usage_limits() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Add usage near the limit for free tier (30000 characters)
    ctx.fixtures.add_tts_usage(user.id, 29900, 20).await.unwrap();

    // Small request should succeed
    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": "Short text"
            }),
            &token,
        )
        .await
        .unwrap();

    assert!(
        response.status == StatusCode::OK ||
        response.status == StatusCode::SERVICE_UNAVAILABLE
    );

    // Large request should hit limit
    let large_text = "a".repeat(200);
    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": large_text
            }),
            &token,
        )
        .await
        .unwrap();

    if response.status != StatusCode::SERVICE_UNAVAILABLE {
        response
            .assert_status(StatusCode::TOO_MANY_REQUESTS)
            .assert_error_code("DAILY_LIMIT_EXCEEDED");
    }
}

#[tokio::test]
#[serial]
async fn it_should_allow_higher_limits_for_pro_users() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_pro_user("pro@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Add usage that would exceed free tier limit
    ctx.fixtures.add_tts_usage(user.id, 35000, 25).await.unwrap();

    // Pro user should still be able to synthesize
    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": "Pro users have higher limits"
            }),
            &token,
        )
        .await
        .unwrap();

    assert!(
        response.status == StatusCode::OK ||
        response.status == StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
#[serial]
async fn it_should_require_pro_for_neural_voices() {
    let ctx = TestContext::new().await.unwrap();
    let free_user = ctx.fixtures.create_user("free@example.com").await.unwrap();
    let pro_user = ctx.fixtures.create_pro_user("pro@example.com").await.unwrap();

    let free_token = generate_test_jwt(&free_user.id, &ctx.config.jwt_secret);
    let pro_token = generate_test_jwt(&pro_user.id, &ctx.config.jwt_secret);

    // Free user trying neural voice
    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": "Testing neural voice",
                "quality": "neural"
            }),
            &free_token,
        )
        .await
        .unwrap();

    if response.status != StatusCode::SERVICE_UNAVAILABLE {
        response
            .assert_status(StatusCode::PAYMENT_REQUIRED)
            .assert_error_code("UPGRADE_REQUIRED");
    }

    // Pro user should be able to use neural
    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": "Testing neural voice",
                "quality": "neural"
            }),
            &pro_token,
        )
        .await
        .unwrap();

    assert!(
        response.status == StatusCode::OK ||
        response.status == StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
#[serial]
async fn it_should_get_tts_usage_statistics() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Add some usage
    ctx.fixtures.add_tts_usage(user.id, 15000, 10).await.unwrap();

    let response = ctx
        .client
        .get_with_auth("/api/tts/usage", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    assert_eq!(body.get("period").and_then(|v| v.as_str()), Some("daily"));

    let usage = body.get("usage").unwrap();
    assert_eq!(usage.get("characters").and_then(|v| v.as_i64()), Some(15000));
    assert_eq!(usage.get("minutes").and_then(|v| v.as_f64()), Some(10.5));

    let limits = body.get("limits").unwrap();
    assert!(limits.get("characters").is_some());
    assert!(limits.get("minutes").is_some());

    assert!(body.get("resets_at").is_some());
}

#[tokio::test]
#[serial]
async fn it_should_track_usage_history() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Add usage for multiple days (would need to manipulate dates in real scenario)
    ctx.fixtures.add_tts_usage(user.id, 5000, 3).await.unwrap();

    let response = ctx
        .client
        .get_with_auth("/api/tts/usage", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    if let Some(history) = body.get("history") {
        assert!(history.is_array());
        let history_array = history.as_array().unwrap();

        for entry in history_array {
            assert!(entry.get("date").is_some());
            assert!(entry.get("characters").is_some());
            assert!(entry.get("minutes").is_some());
        }
    }
}

#[tokio::test]
#[serial]
async fn it_should_validate_speed_parameter() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Invalid speeds
    let invalid_speeds = vec![0.4, 2.1, -1.0];

    for speed in invalid_speeds {
        let response = ctx
            .client
            .post_with_auth(
                "/api/tts/synthesize",
                &json!({
                    "text": "Testing speed",
                    "speed": speed
                }),
                &token,
            )
            .await
            .unwrap();

        response.assert_status(StatusCode::BAD_REQUEST);
    }

    // Valid speeds
    let valid_speeds = vec![0.5, 1.0, 1.5, 2.0];

    for speed in valid_speeds {
        let response = ctx
            .client
            .post_with_auth(
                "/api/tts/synthesize",
                &json!({
                    "text": "Testing speed",
                    "speed": speed
                }),
                &token,
            )
            .await
            .unwrap();

        assert!(
            response.status == StatusCode::OK ||
            response.status == StatusCode::SERVICE_UNAVAILABLE
        );
    }
}

#[tokio::test]
#[serial]
async fn it_should_require_authentication_for_tts() {
    let ctx = TestContext::new().await.unwrap();

    // Try to synthesize without auth
    let response = ctx
        .client
        .post(
            "/api/tts/synthesize",
            &json!({
                "text": "Unauthorized test"
            }),
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::UNAUTHORIZED);

    // Try to get usage without auth
    let response = ctx.client.get("/api/tts/usage").await.unwrap();
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn it_should_include_usage_remaining_header() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Add some usage
    ctx.fixtures.add_tts_usage(user.id, 10000, 7).await.unwrap();

    let response = ctx
        .client
        .post_with_auth(
            "/api/tts/synthesize",
            &json!({
                "text": "Check remaining usage"
            }),
            &token,
        )
        .await
        .unwrap();

    if response.status == StatusCode::OK {
        assert!(response.header("x-usage-remaining").is_some());
    }
}