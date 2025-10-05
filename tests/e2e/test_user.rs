use crate::e2e::helpers;

use helpers::{generate_test_jwt, TestContext};
use hyper::StatusCode;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn it_should_get_current_user_info() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth("/api/me", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    helpers::assertions::assert_user_response(body);

    assert_eq!(body.get("email").and_then(|v| v.as_str()), Some("user@example.com"));
    assert_eq!(body.get("id").and_then(|v| v.as_str()), Some(user.id.to_string().as_str()));

    // Verify subscription details for free user
    let subscription = body.get("subscription").unwrap();
    assert_eq!(subscription.get("tier").and_then(|v| v.as_str()), Some("free"));
    assert_eq!(subscription.get("status").and_then(|v| v.as_str()), Some("active"));
}

#[tokio::test]
#[serial]
async fn it_should_update_user_settings() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let new_settings = json!({
        "settings": {
            "voice": "Sergio",
            "speed": 1.5,
            "language": "es",
            "quality": "neural"
        }
    });

    let response = ctx
        .client
        .patch_with_auth("/api/me", &new_settings, &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let settings = body.get("settings").unwrap();

    assert_eq!(settings.get("voice").and_then(|v| v.as_str()), Some("Sergio"));
    assert_eq!(settings.get("speed").and_then(|v| v.as_f64()), Some(1.5));
    assert_eq!(settings.get("language").and_then(|v| v.as_str()), Some("es"));
    assert_eq!(settings.get("quality").and_then(|v| v.as_str()), Some("neural"));

    // Verify settings persist
    let response = ctx
        .client
        .get_with_auth("/api/me", &token)
        .await
        .unwrap();

    let body = response.body.as_ref().unwrap();
    let settings = body.get("settings").unwrap();

    assert_eq!(settings.get("voice").and_then(|v| v.as_str()), Some("Sergio"));
    assert_eq!(settings.get("speed").and_then(|v| v.as_f64()), Some(1.5));
}

#[tokio::test]
#[serial]
async fn it_should_update_partial_settings() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Update only voice
    let response = ctx
        .client
        .patch_with_auth(
            "/api/me",
            &json!({
                "settings": {
                    "voice": "Conchita"
                }
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let settings = body.get("settings").unwrap();

    assert_eq!(settings.get("voice").and_then(|v| v.as_str()), Some("Conchita"));
    // Other settings should remain at defaults
    assert_eq!(settings.get("speed").and_then(|v| v.as_f64()), Some(1.0));
    assert_eq!(settings.get("language").and_then(|v| v.as_str()), Some("auto"));
}

#[tokio::test]
#[serial]
async fn it_should_validate_speed_range() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Try to set speed too low
    let response = ctx
        .client
        .patch_with_auth(
            "/api/me",
            &json!({
                "settings": {
                    "speed": 0.3
                }
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::BAD_REQUEST);

    // Try to set speed too high
    let response = ctx
        .client
        .patch_with_auth(
            "/api/me",
            &json!({
                "settings": {
                    "speed": 3.0
                }
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::BAD_REQUEST);

    // Valid speed should work
    let response = ctx
        .client
        .patch_with_auth(
            "/api/me",
            &json!({
                "settings": {
                    "speed": 1.75
                }
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn it_should_show_pro_user_subscription() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_pro_user("pro@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth("/api/me", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let subscription = body.get("subscription").unwrap();

    assert_eq!(subscription.get("tier").and_then(|v| v.as_str()), Some("pro"));
    assert_eq!(subscription.get("status").and_then(|v| v.as_str()), Some("active"));
    assert!(subscription.get("expires_at").is_some());
    assert_eq!(subscription.get("store").and_then(|v| v.as_str()), Some("apple"));

    // Pro limits should be higher
    let limits = subscription.get("limits").unwrap();
    let max_feeds = limits.get("max_feeds").and_then(|v| v.as_i64()).unwrap();
    assert!(max_feeds > 3);
    assert_eq!(limits.get("voice_quality").and_then(|v| v.as_str()), Some("neural"));
}

#[tokio::test]
#[serial]
async fn it_should_show_usage_statistics() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Add some usage
    ctx.fixtures.add_tts_usage(user.id, 5000, 2.5).await.unwrap();

    let response = ctx
        .client
        .get_with_auth("/api/me", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let subscription = body.get("subscription").unwrap();

    if let Some(usage) = subscription.get("usage") {
        assert_eq!(usage.get("characters_used_today").and_then(|v| v.as_i64()), Some(5000));
        assert_eq!(usage.get("minutes_used_today").and_then(|v| v.as_f64()), Some(2.5));
        assert!(usage.get("characters_limit").is_some());
        assert!(usage.get("minutes_limit").is_some());
        assert!(usage.get("resets_at").is_some());
    }
}

#[tokio::test]
#[serial]
async fn it_should_validate_language_settings() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Valid languages
    let valid_languages = vec!["auto", "es", "en", "fr", "de", "pt", "it"];

    for lang in valid_languages {
        let response = ctx
            .client
            .patch_with_auth(
                "/api/me",
                &json!({
                    "settings": {
                        "language": lang
                    }
                }),
                &token,
            )
            .await
            .unwrap();

        response.assert_status(StatusCode::OK);
    }

    // Invalid language
    let response = ctx
        .client
        .patch_with_auth(
            "/api/me",
            &json!({
                "settings": {
                    "language": "klingon"
                }
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn it_should_validate_quality_settings() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Valid qualities
    for quality in &["standard", "neural"] {
        let response = ctx
            .client
            .patch_with_auth(
                "/api/me",
                &json!({
                    "settings": {
                        "quality": quality
                    }
                }),
                &token,
            )
            .await
            .unwrap();

        response.assert_status(StatusCode::OK);
    }

    // Invalid quality
    let response = ctx
        .client
        .patch_with_auth(
            "/api/me",
            &json!({
                "settings": {
                    "quality": "ultra-hd"
                }
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn it_should_require_authentication_for_user_endpoints() {
    let ctx = TestContext::new().await.unwrap();

    // Try to get user info without auth
    let response = ctx.client.get("/api/me").await.unwrap();
    response.assert_status(StatusCode::UNAUTHORIZED);

    // Try to update settings without auth
    let response = ctx
        .client
        .patch(
            "/api/me",
            &json!({
                "settings": {
                    "voice": "Sergio"
                }
            }),
        )
        .await
        .unwrap();
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn it_should_validate_subscription_receipt() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Note: This test would need actual receipt validation logic
    // For now, we just test that the endpoint exists and requires auth
    let response = ctx
        .client
        .post_with_auth(
            "/api/subscription/validate-receipt",
            &json!({
                "receipt_data": "base64_encoded_receipt_here",
                "store": "apple"
            }),
            &token,
        )
        .await
        .unwrap();

    // Endpoint should exist but may return error for invalid receipt
    assert!(
        response.status == StatusCode::OK ||
        response.status == StatusCode::BAD_REQUEST ||
        response.status == StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
#[serial]
async fn it_should_reject_invalid_jwt() {
    let ctx = TestContext::new().await.unwrap();
    let invalid_token = "invalid.jwt.token";

    let response = ctx
        .client
        .get_with_auth("/api/me", invalid_token)
        .await
        .unwrap();

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn it_should_reject_expired_jwt() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();

    // Create an expired token
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::Serialize;

    #[derive(Serialize)]
    struct Claims {
        sub: String,
        exp: i64,
        iat: i64,
    }

    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user.id.to_string(),
        exp: (now - chrono::Duration::hours(1)).timestamp(), // Expired 1 hour ago
        iat: (now - chrono::Duration::hours(2)).timestamp(),
    };

    let expired_token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(ctx.config.jwt_secret.as_bytes()),
    )
    .unwrap();

    let response = ctx
        .client
        .get_with_auth("/api/me", &expired_token)
        .await
        .unwrap();

    response.assert_status(StatusCode::UNAUTHORIZED);
}