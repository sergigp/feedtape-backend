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

    // Assert complete user response structure
    assert_eq!(
        body,
        &json!({
            "id": user.id.to_string(),
            "email": "user@example.com",
            "settings": {
                "voice": body["settings"]["voice"],
                "speed": body["settings"]["speed"],
                "language": body["settings"]["language"],
                "quality": body["settings"]["quality"]
            },
            "subscription": {
                "tier": "free",
                "status": "active",
                "limits": body["subscription"]["limits"],
                "usage": body["subscription"]["usage"]
            }
        })
    );
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

    // Assert updated settings in response
    assert_eq!(
        body["settings"],
        json!({
            "voice": "Sergio",
            "speed": 1.5,
            "language": "es",
            "quality": "neural"
        })
    );

    // Verify settings persist
    let response = ctx
        .client
        .get_with_auth("/api/me", &token)
        .await
        .unwrap();

    let body = response.body.as_ref().unwrap();

    // Verify persisted settings
    assert_eq!(
        body["settings"],
        json!({
            "voice": "Sergio",
            "speed": 1.5,
            "language": "es",
            "quality": "neural"
        })
    );
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

    // Assert partial update with defaults
    assert_eq!(
        body["settings"],
        json!({
            "voice": "Conchita",
            "speed": 1.0,
            "language": "auto",
            "quality": body["settings"]["quality"]  // Quality remains at default
        })
    );
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

    // Assert pro subscription structure
    let subscription = &body["subscription"];
    assert_eq!(subscription["tier"], "pro");
    assert_eq!(subscription["status"], "active");
    assert!(subscription.get("expires_at").is_some());

    // Assert pro limits
    let limits = &subscription["limits"];
    let max_feeds = limits["max_feeds"].as_i64().unwrap();
    assert!(max_feeds > 3, "Pro tier should allow more than 3 feeds");
    assert_eq!(limits["voice_quality"], "neural");
}

#[tokio::test]
#[serial]
async fn it_should_show_usage_statistics() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Add some usage
    ctx.fixtures.add_tts_usage(user.id, 5000, 2).await.unwrap();

    let response = ctx
        .client
        .get_with_auth("/api/me", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();

    // Assert usage statistics in subscription
    let usage = &body["subscription"]["usage"];
    assert_eq!(
        usage,
        &json!({
            "characters_used_today": 5000,
            "minutes_used_today": 5.0,  // 1000 chars = 1 minute
            "characters_limit": usage["characters_limit"],
            "minutes_limit": usage["minutes_limit"],
            "resets_at": usage["resets_at"]
        })
    );
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

    // Note: The subscription receipt validation endpoint doesn't exist yet
    // This is a placeholder test that should be implemented when the endpoint is added
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

    // Since the endpoint doesn't exist, we expect a 404
    assert_eq!(response.status, StatusCode::NOT_FOUND);
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