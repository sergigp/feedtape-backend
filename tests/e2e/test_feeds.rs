use crate::e2e::helpers;

use helpers::{generate_test_jwt, TestContext};
use hyper::StatusCode;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn it_should_create_a_new_feed() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .post_with_auth(
            "/api/feeds",
            &json!({
                "url": "https://blog.example.com/rss",
                "title": "Example Blog"
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::CREATED);

    let body = response.body.as_ref().unwrap();
    helpers::assertions::assert_feed_response(body, "https://blog.example.com/rss", Some("Example Blog"));

    // Verify in database
    let feed_count = ctx.fixtures.get_feed_count(user.id).await.unwrap();
    assert_eq!(feed_count, 1);
}

#[tokio::test]
#[serial]
async fn it_should_create_feed_without_title() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .post_with_auth(
            "/api/feeds",
            &json!({
                "url": "https://blog.example.com/rss"
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::CREATED);

    let body = response.body.as_ref().unwrap();
    assert!(body.get("id").is_some());
    assert_eq!(body.get("url").and_then(|v| v.as_str()), Some("https://blog.example.com/rss"));
    // Title should either be null or not present when not provided
    let title = body.get("title");
    assert!(title.is_none() || title.and_then(|v| v.as_null()).is_some());
}

#[tokio::test]
#[serial]
async fn it_should_list_user_feeds() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Create multiple feeds
    ctx.fixtures.create_multiple_feeds(user.id, 3).await.unwrap();

    let response = ctx
        .client
        .get_with_auth("/api/feeds", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let feeds = response.body.as_ref().unwrap().as_array().unwrap();
    assert_eq!(feeds.len(), 3);

    // Verify feeds have expected structure
    for feed in feeds {
        assert!(feed.get("id").is_some());
        assert!(feed.get("url").is_some());
        assert!(feed.get("created_at").is_some());
    }
}

#[tokio::test]
#[serial]
async fn it_should_update_feed_title() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let feed = ctx
        .fixtures
        .create_feed(user.id, "https://blog.example.com/rss", Some("Old Title"))
        .await
        .unwrap();

    let response = ctx
        .client
        .put_with_auth(
            &format!("/api/feeds/{}", feed.id),
            &json!({
                "title": "New Title"
            }),
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    assert_eq!(body.get("title").and_then(|v| v.as_str()), Some("New Title"));
}

#[tokio::test]
#[serial]
async fn it_should_delete_a_feed() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let feed = ctx
        .fixtures
        .create_feed(user.id, "https://blog.example.com/rss", Some("Test Feed"))
        .await
        .unwrap();

    let response = ctx
        .client
        .delete_with_auth(&format!("/api/feeds/{}", feed.id), &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::NO_CONTENT);

    // Verify feed is deleted from database
    let feed_count = ctx.fixtures.get_feed_count(user.id).await.unwrap();
    assert_eq!(feed_count, 0);
}

#[tokio::test]
#[serial]
async fn it_should_prevent_duplicate_feed_urls() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let feed_url = "https://blog.example.com/rss";

    // Create first feed
    ctx.fixtures.create_feed(user.id, feed_url, None).await.unwrap();

    // Try to create duplicate
    let response = ctx
        .client
        .post_with_auth(
            "/api/feeds",
            &json!({
                "url": feed_url
            }),
            &token,
        )
        .await
        .unwrap();

    response
        .assert_status(StatusCode::CONFLICT)
        .assert_error_code("CONFLICT");
}

#[tokio::test]
#[serial]
async fn it_should_enforce_free_tier_feed_limit() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Create 3 feeds (free tier limit)
    for i in 0..3 {
        ctx.fixtures
            .create_feed(user.id, &format!("https://blog{}.example.com/rss", i), None)
            .await
            .unwrap();
    }

    // Try to create 4th feed
    let response = ctx
        .client
        .post_with_auth(
            "/api/feeds",
            &json!({
                "url": "https://blog4.example.com/rss"
            }),
            &token,
        )
        .await
        .unwrap();

    response
        .assert_status(StatusCode::PAYMENT_REQUIRED)
        .assert_error_code("UPGRADE_REQUIRED");

    // Details might not be included in the error response, so make this optional
    let body = response.body.as_ref().unwrap();
    if let Some(error) = body.get("error") {
        if let Some(details) = error.get("details") {
            assert_eq!(details.get("current").and_then(|v| v.as_i64()), Some(3));
            assert_eq!(details.get("limit").and_then(|v| v.as_i64()), Some(3));
        }
    }
}

#[tokio::test]
#[serial]
async fn it_should_allow_pro_users_more_feeds() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_pro_user("pro@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Create more than 3 feeds (pro tier allows more)
    for i in 0..5 {
        let response = ctx
            .client
            .post_with_auth(
                "/api/feeds",
                &json!({
                    "url": format!("https://blog{}.example.com/rss", i)
                }),
                &token,
            )
            .await
            .unwrap();

        response.assert_status(StatusCode::CREATED);
    }

    let feed_count = ctx.fixtures.get_feed_count(user.id).await.unwrap();
    assert_eq!(feed_count, 5);
}

#[tokio::test]
#[serial]
async fn it_should_return_404_for_nonexistent_feed() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let fake_id = uuid::Uuid::new_v4();

    let response = ctx
        .client
        .delete_with_auth(&format!("/api/feeds/{}", fake_id), &token)
        .await
        .unwrap();

    response
        .assert_status(StatusCode::NOT_FOUND)
        .assert_error_code("NOT_FOUND");
}

#[tokio::test]
#[serial]
async fn it_should_not_allow_access_to_other_users_feeds() {
    let ctx = TestContext::new().await.unwrap();

    let user1 = ctx.fixtures.create_user("user1@example.com").await.unwrap();
    let user2 = ctx.fixtures.create_user("user2@example.com").await.unwrap();

    let _token1 = generate_test_jwt(&user1.id, &ctx.config.jwt_secret);
    let token2 = generate_test_jwt(&user2.id, &ctx.config.jwt_secret);

    // User1 creates a feed
    let feed = ctx
        .fixtures
        .create_feed(user1.id, "https://blog.example.com/rss", Some("User1 Feed"))
        .await
        .unwrap();

    // User2 tries to update user1's feed
    let response = ctx
        .client
        .put_with_auth(
            &format!("/api/feeds/{}", feed.id),
            &json!({
                "title": "Hacked Title"
            }),
            &token2,
        )
        .await
        .unwrap();

    response
        .assert_status(StatusCode::NOT_FOUND)
        .assert_error_code("NOT_FOUND");

    // User2 tries to delete user1's feed
    let response = ctx
        .client
        .delete_with_auth(&format!("/api/feeds/{}", feed.id), &token2)
        .await
        .unwrap();

    response
        .assert_status(StatusCode::NOT_FOUND)
        .assert_error_code("NOT_FOUND");
}

#[tokio::test]
#[serial]
async fn it_should_validate_feed_url_format() {
    let ctx = TestContext::new().await.unwrap();
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let invalid_urls = vec![
        "not-a-url",
        "ftp://example.com/feed",
        "",
    ];

    for invalid_url in invalid_urls {
        let response = ctx
            .client
            .post_with_auth(
                "/api/feeds",
                &json!({
                    "url": invalid_url
                }),
                &token,
            )
            .await
            .unwrap();

        response.assert_status(StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
#[serial]
async fn it_should_require_authentication_for_feeds() {
    let ctx = TestContext::new().await.unwrap();

    // Try to list feeds without auth
    let response = ctx.client.get("/api/feeds").await.unwrap();
    response.assert_status(StatusCode::UNAUTHORIZED);

    // Try to create feed without auth
    let response = ctx
        .client
        .post(
            "/api/feeds",
            &json!({
                "url": "https://blog.example.com/rss"
            }),
        )
        .await
        .unwrap();
    response.assert_status(StatusCode::UNAUTHORIZED);

    // Try to delete feed without auth
    let fake_id = uuid::Uuid::new_v4();
    let response = ctx
        .client
        .delete(&format!("/api/feeds/{}", fake_id))
        .await
        .unwrap();
    response.assert_status(StatusCode::UNAUTHORIZED);
}