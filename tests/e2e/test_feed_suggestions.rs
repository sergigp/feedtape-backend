use crate::e2e::helpers;

use helpers::{generate_test_jwt, TestContext};
use hyper::StatusCode;
use test_context::test_context;
use std::collections::HashSet;

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_get_all_categories_with_suggestions(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth("/api/feed-suggestions", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    assert_eq!(
        categories.len(),
        20,
        "Should return all 20 categories when no filter provided"
    );

    // Verify structure of each category
    for category in categories {
        assert!(category["id"].is_string());
        assert!(category["name"].is_string());
        assert!(category["description"].is_string());
        assert!(category["suggestions"].is_array());

        let suggestions = category["suggestions"].as_array().unwrap();

        // Each category should have at least 3 suggestions
        assert!(
            suggestions.len() >= 3,
            "Category '{}' should have at least 3 suggestions, got {}",
            category["name"].as_str().unwrap(),
            suggestions.len()
        );

        // Each suggestion should have the correct structure
        for suggestion in suggestions {
            assert!(suggestion["id"].is_string());
            assert!(suggestion["title"].is_string());
            assert!(suggestion["description"].is_string());
            assert!(suggestion["url"].is_string());
            // Verify category field is NOT present (removed from nested structure)
            assert!(
                suggestion["category"].is_null(),
                "Suggestion should not have category field"
            );
        }
    }
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_get_suggestions_by_single_category(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth(
            "/api/feed-suggestions?category_ids=technology-programming",
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    assert_eq!(
        categories.len(),
        1,
        "Should return exactly 1 category for single filter"
    );

    let category = &categories[0];
    assert_eq!(category["id"], "technology-programming");
    assert_eq!(category["name"], "💻 Technology & Programming");

    let suggestions = category["suggestions"].as_array().unwrap();
    assert_eq!(
        suggestions.len(),
        4,
        "Should return 4 suggestions for technology-programming"
    );

    // Verify structure of each suggestion
    for suggestion in suggestions {
        assert!(suggestion["id"].is_string());
        assert!(suggestion["title"].is_string());
        assert!(suggestion["description"].is_string());
        assert!(suggestion["url"].is_string());
    }
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_get_suggestions_by_multiple_categories(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth(
            "/api/feed-suggestions?category_ids=technology-programming,science-research",
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    assert_eq!(
        categories.len(),
        2,
        "Should return 2 categories for multiple filters"
    );

    // Verify both categories are present
    let category_ids: Vec<String> = categories
        .iter()
        .map(|c| c["id"].as_str().unwrap().to_string())
        .collect();

    assert!(category_ids.contains(&"technology-programming".to_string()));
    assert!(category_ids.contains(&"science-research".to_string()));

    // Count total suggestions
    let total_suggestions: usize = categories
        .iter()
        .map(|c| c["suggestions"].as_array().unwrap().len())
        .sum();

    assert_eq!(
        total_suggestions, 8,
        "Should have 8 total suggestions (4 from each category)"
    );
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_return_empty_for_invalid_category_ids(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth(
            "/api/feed-suggestions?category_ids=invalid-category,nonexistent",
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    assert_eq!(
        categories.len(),
        0,
        "Should return empty array for invalid category IDs"
    );
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_require_authentication(ctx: &TestContext) {

    // Try without auth token
    let response = ctx
        .client
        .get("/api/feed-suggestions?category_ids=technology-programming")
        .await
        .unwrap();

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_return_categories_with_all_fields(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth("/api/feed-suggestions", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    // Verify each category has all required fields
    for category in categories {
        assert!(category["id"].is_string(), "Category should have id");
        assert!(category["name"].is_string(), "Category should have name");
        assert!(
            category["description"].is_string(),
            "Category should have description"
        );
        assert!(
            category["suggestions"].is_array(),
            "Category should have suggestions array"
        );

        let id = category["id"].as_str().unwrap();
        let name = category["name"].as_str().unwrap();
        let description = category["description"].as_str().unwrap();
        let suggestions = category["suggestions"].as_array().unwrap();

        assert!(!id.is_empty(), "Category ID should not be empty");
        assert!(!name.is_empty(), "Category name should not be empty");
        assert!(
            description.len() >= 50,
            "Category description should be at least 50 characters"
        );
        assert!(
            suggestions.len() >= 3,
            "Category '{}' should have at least 3 suggestions, got {}",
            name,
            suggestions.len()
        );
    }
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_sort_categories_alphabetically(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth("/api/feed-suggestions", &token)
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    // Extract names
    let names: Vec<String> = categories
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect();

    // Verify alphabetical order
    let mut sorted_names = names.clone();
    sorted_names.sort();

    assert_eq!(
        names, sorted_names,
        "Categories should be sorted alphabetically by name"
    );
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_allow_existing_users_to_browse_suggestions(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Create some feeds for the user (simulating existing user)
    ctx.fixtures
        .create_feed(user.id, "https://blog1.example.com/rss", Some("Blog 1"))
        .await
        .unwrap();
    ctx.fixtures
        .create_feed(user.id, "https://blog2.example.com/rss", Some("Blog 2"))
        .await
        .unwrap();

    // Now access suggestions endpoint
    let response = ctx
        .client
        .get_with_auth(
            "/api/feed-suggestions?category_ids=technology-programming",
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    assert_eq!(categories.len(), 1);

    let suggestions = categories[0]["suggestions"].as_array().unwrap();
    assert_eq!(
        suggestions.len(),
        4,
        "Existing users should still get suggestions"
    );
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_return_consistent_suggestions_for_same_category(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Call endpoint twice
    let response1 = ctx
        .client
        .get_with_auth(
            "/api/feed-suggestions?category_ids=technology-programming",
            &token,
        )
        .await
        .unwrap();

    let response2 = ctx
        .client
        .get_with_auth(
            "/api/feed-suggestions?category_ids=technology-programming",
            &token,
        )
        .await
        .unwrap();

    response1.assert_status(StatusCode::OK);
    response2.assert_status(StatusCode::OK);

    let categories1 = response1.body.as_ref().unwrap()["categories"]
        .as_array()
        .unwrap();
    let categories2 = response2.body.as_ref().unwrap()["categories"]
        .as_array()
        .unwrap();

    let suggestions1 = categories1[0]["suggestions"].as_array().unwrap();
    let suggestions2 = categories2[0]["suggestions"].as_array().unwrap();

    // Extract URLs from both responses
    let urls1: Vec<String> = suggestions1
        .iter()
        .map(|s| s["url"].as_str().unwrap().to_string())
        .collect();
    let urls2: Vec<String> = suggestions2
        .iter()
        .map(|s| s["url"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(
        urls1, urls2,
        "Should return consistent suggestions for the same category"
    );
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_support_categories_alias_param(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    // Use "categories" instead of "category_ids"
    let response = ctx
        .client
        .get_with_auth(
            "/api/feed-suggestions?categories=technology-programming",
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    assert_eq!(categories.len(), 1);

    let suggestions = categories[0]["suggestions"].as_array().unwrap();
    assert_eq!(
        suggestions.len(),
        4,
        "Should work with 'categories' parameter alias"
    );
}

#[test_context(TestContext)]
#[tokio::test]
async fn it_should_have_unique_urls_across_all_suggestions(ctx: &TestContext) {
    let user = ctx.fixtures.create_user("user@example.com").await.unwrap();
    let token = generate_test_jwt(&user.id, &ctx.config.jwt_secret);

    let response = ctx
        .client
        .get_with_auth(
            "/api/feed-suggestions?category_ids=technology-programming,science-research",
            &token,
        )
        .await
        .unwrap();

    response.assert_status(StatusCode::OK);

    let body = response.body.as_ref().unwrap();
    let categories = body["categories"].as_array().unwrap();

    // Extract all URLs from all categories
    let mut all_urls: Vec<String> = Vec::new();
    for category in categories {
        let suggestions = category["suggestions"].as_array().unwrap();
        for suggestion in suggestions {
            all_urls.push(suggestion["url"].as_str().unwrap().to_string());
        }
    }

    // Check for duplicates
    let unique_urls: HashSet<_> = all_urls.iter().collect();
    assert_eq!(
        all_urls.len(),
        unique_urls.len(),
        "Should have no duplicate URLs across categories"
    );
}
