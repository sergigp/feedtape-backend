use serde_json::Value;

pub fn assert_feed_response(feed: &Value, expected_url: &str, expected_title: Option<&str>) {
    assert!(feed.get("id").and_then(|v| v.as_str()).is_some());
    assert_eq!(
        feed.get("url").and_then(|v| v.as_str()),
        Some(expected_url)
    );

    if let Some(title) = expected_title {
        assert_eq!(
            feed.get("title").and_then(|v| v.as_str()),
            Some(title)
        );
    }

    assert!(feed.get("created_at").is_some());
}

pub fn assert_user_response(user: &Value) {
    // Basic user fields
    assert!(user.get("id").and_then(|v| v.as_str()).is_some());
    assert!(user.get("email").and_then(|v| v.as_str()).is_some());

    // Settings
    let settings = user.get("settings").expect("Missing settings");
    assert!(settings.get("voice").is_some());
    assert!(settings.get("speed").is_some());
    assert!(settings.get("language").is_some());
    assert!(settings.get("quality").is_some());

    // Subscription
    let subscription = user.get("subscription").expect("Missing subscription");
    assert!(subscription.get("tier").is_some());
    assert!(subscription.get("status").is_some());

    // Usage (if present)
    if let Some(usage) = subscription.get("usage") {
        assert!(usage.get("characters_used_today").is_some());
        assert!(usage.get("characters_limit").is_some());
        assert!(usage.get("minutes_used_today").is_some());
        assert!(usage.get("minutes_limit").is_some());
    }

    // Limits
    if let Some(limits) = subscription.get("limits") {
        assert!(limits.get("max_feeds").is_some());
        assert!(limits.get("voice_quality").is_some());
    }
}

pub fn assert_token_response(response: &Value) {
    assert!(
        response.get("token").and_then(|v| v.as_str()).is_some(),
        "Missing token field"
    );
    assert!(
        response
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .is_some(),
        "Missing refresh_token field"
    );
    assert!(
        response.get("expires_in").and_then(|v| v.as_i64()).is_some(),
        "Missing expires_in field"
    );
}

#[allow(dead_code)]
pub fn assert_error_response(response: &Value, expected_code: &str) {
    let error = response.get("error").expect("Missing error field");
    let code = error.get("code").and_then(|v| v.as_str()).unwrap();
    assert_eq!(code, expected_code, "Error code mismatch");
    assert!(error.get("message").is_some(), "Missing error message");
    assert!(response.get("request_id").is_some(), "Missing request_id");
}

#[allow(dead_code)]
pub fn assert_tts_headers(headers: &std::collections::HashMap<String, String>) {
    assert!(
        headers.contains_key("content-type"),
        "Missing Content-Type header"
    );
    assert!(
        headers.contains_key("x-character-count"),
        "Missing X-Character-Count header"
    );
    assert!(
        headers.contains_key("x-voice-used"),
        "Missing X-Voice-Used header"
    );
}