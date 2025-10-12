# Quickstart: Feed Suggestions Implementation

**Feature**: Feed Suggestions by Category
**Date**: 2025-10-11
**Prerequisites**: Rust 1.70+, Docker, existing FeedTape Backend setup

## Overview

This guide walks through implementing the feed suggestions feature from scratch. Follow these steps to add curated RSS feed recommendations organized by category.

## Step 1: Create Domain Module

### 1.1 Create directory structure

```bash
mkdir -p src/domain/feed_suggestions
touch src/domain/feed_suggestions/mod.rs
touch src/domain/feed_suggestions/service.rs
```

### 1.2 Define entities in `src/domain/feed_suggestions/mod.rs`

```rust
use serde::{Deserialize, Serialize};

/// Represents a content category for organizing feed suggestions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Represents a curated RSS feed recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSuggestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub category_id: String,
}

/// Repository trait for accessing feed suggestions data
pub trait FeedSuggestionsRepository: Send + Sync {
    fn get_all_categories(&self) -> Vec<Category>;
    fn get_suggestions_by_categories(&self, category_ids: &[String]) -> Vec<FeedSuggestion>;
}

// Re-export service
pub mod service;
pub use service::FeedSuggestionsService;
```

### 1.3 Implement service in `src/domain/feed_suggestions/service.rs`

```rust
use super::{Category, FeedSuggestion, FeedSuggestionsRepository};
use std::sync::Arc;

pub struct FeedSuggestionsService {
    repository: Arc<dyn FeedSuggestionsRepository>,
}

impl FeedSuggestionsService {
    pub fn new(repository: Arc<dyn FeedSuggestionsRepository>) -> Self {
        Self { repository }
    }

    pub fn get_categories(&self) -> Vec<Category> {
        self.repository.get_all_categories()
    }

    pub fn get_suggestions(&self, category_ids: Vec<String>) -> Vec<FeedSuggestion> {
        if category_ids.is_empty() {
            return Vec::new();
        }

        self.repository.get_suggestions_by_categories(&category_ids)
    }
}
```

### 1.4 Register module in `src/domain/mod.rs`

```rust
pub mod feed_suggestions;
```

---

## Step 2: Create Repository Implementation

### 2.1 Create repository file

```bash
touch src/infrastructure/repositories/feed_suggestions_repository.rs
```

### 2.2 Implement hardcoded repository

```rust
use crate::domain::feed_suggestions::{Category, FeedSuggestion, FeedSuggestionsRepository};
use std::collections::HashSet;
use std::sync::LazyLock;

static CATEGORIES: LazyLock<Vec<Category>> = LazyLock::new(|| {
    vec![
        Category {
            id: "news-current-affairs".to_string(),
            name: "📰 News & Current Affairs".to_string(),
            description: "Stay informed with breaking news and in-depth analysis from trusted sources".to_string(),
        },
        Category {
            id: "technology-programming".to_string(),
            name: "💻 Technology & Programming".to_string(),
            description: "Latest in tech, programming languages, frameworks, and software development".to_string(),
        },
        // ... Add all 20 categories (see data-model.md for complete list)
    ]
});

static FEED_SUGGESTIONS: LazyLock<Vec<FeedSuggestion>> = LazyLock::new(|| {
    vec![
        // News & Current Affairs (4 feeds)
        FeedSuggestion {
            id: "bbc-news".to_string(),
            title: "BBC News".to_string(),
            description: "Breaking news, analysis and features from the BBC with global coverage".to_string(),
            url: "https://feeds.bbci.co.uk/news/rss.xml".to_string(),
            category_id: "news-current-affairs".to_string(),
        },
        FeedSuggestion {
            id: "the-guardian".to_string(),
            title: "The Guardian".to_string(),
            description: "Independent journalism covering news, politics, culture, and sport".to_string(),
            url: "https://www.theguardian.com/rss".to_string(),
            category_id: "news-current-affairs".to_string(),
        },
        // ... Add all 80 suggestions (4 per category, see user's provided list)
    ]
});

pub struct HardcodedFeedSuggestionsRepository;

impl HardcodedFeedSuggestionsRepository {
    pub fn new() -> Self {
        Self
    }
}

impl FeedSuggestionsRepository for HardcodedFeedSuggestionsRepository {
    fn get_all_categories(&self) -> Vec<Category> {
        CATEGORIES.clone()
    }

    fn get_suggestions_by_categories(&self, category_ids: &[String]) -> Vec<FeedSuggestion> {
        let mut seen_urls = HashSet::new();
        let mut results = Vec::new();

        for suggestion in FEED_SUGGESTIONS.iter() {
            if category_ids.contains(&suggestion.category_id)
                && seen_urls.insert(&suggestion.url) {
                results.push(suggestion.clone());
            }
        }

        results
    }
}
```

### 2.3 Register repository in `src/infrastructure/repositories/mod.rs`

```rust
pub mod feed_suggestions_repository;
pub use feed_suggestions_repository::HardcodedFeedSuggestionsRepository;
```

---

## Step 3: Create Controllers

### 3.1 Create controller file

```bash
touch src/controllers/feed_suggestions.rs
```

### 3.2 Implement HTTP handlers

```rust
use crate::domain::feed_suggestions::{Category, FeedSuggestion, FeedSuggestionsService};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct GetSuggestionsQuery {
    #[serde(default)]
    pub category_ids: Option<String>, // Comma-separated
    #[serde(default)]
    pub categories: Option<String>,   // Alias
}

#[derive(Debug, Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<Category>,
}

#[derive(Debug, Serialize)]
pub struct CategoryRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct FeedSuggestionResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub category: CategoryRef,
}

#[derive(Debug, Serialize)]
pub struct SuggestionsResponse {
    pub suggestions: Vec<FeedSuggestionResponse>,
}

// Handlers
pub async fn get_categories(
    State(service): State<Arc<FeedSuggestionsService>>,
) -> impl IntoResponse {
    let categories = service.get_categories();

    (StatusCode::OK, Json(CategoriesResponse { categories }))
}

pub async fn get_suggestions(
    State(service): State<Arc<FeedSuggestionsService>>,
    Query(query): Query<GetSuggestionsQuery>,
) -> impl IntoResponse {
    // Parse category IDs from query params
    let category_ids: Vec<String> = query
        .category_ids
        .or(query.categories)
        .map(|s| s.split(',').map(|id| id.trim().to_string()).collect())
        .unwrap_or_default();

    let suggestions = service.get_suggestions(category_ids);
    let categories = service.get_categories();

    // Build lookup map for category names
    let category_map: std::collections::HashMap<_, _> = categories
        .iter()
        .map(|c| (c.id.clone(), c.name.clone()))
        .collect();

    // Transform to response format with nested category
    let response_suggestions: Vec<FeedSuggestionResponse> = suggestions
        .into_iter()
        .filter_map(|s| {
            category_map.get(&s.category_id).map(|name| {
                FeedSuggestionResponse {
                    id: s.id,
                    title: s.title,
                    description: s.description,
                    url: s.url,
                    category: CategoryRef {
                        id: s.category_id,
                        name: name.clone(),
                    },
                }
            })
        })
        .collect();

    (StatusCode::OK, Json(SuggestionsResponse {
        suggestions: response_suggestions,
    }))
}
```

### 3.3 Register controller in `src/controllers/mod.rs`

```rust
pub mod feed_suggestions;
```

---

## Step 4: Register Routes

### 4.1 Update `src/infrastructure/http/mod.rs`

Add routes under protected API group:

```rust
use crate::controllers::feed_suggestions;
use crate::domain::feed_suggestions::FeedSuggestionsService;

// In your router setup function:
pub fn create_routes(
    feed_suggestions_service: Arc<FeedSuggestionsService>,
    // ... other services
) -> Router {
    // ... existing routes

    let protected_routes = Router::new()
        // ... existing protected routes
        .route("/api/feed-suggestions/categories", get(feed_suggestions::get_categories))
        .route("/api/feed-suggestions", get(feed_suggestions::get_suggestions))
        .with_state(feed_suggestions_service)
        .layer(middleware::from_fn(auth_middleware));

    // ... rest of router
}
```

---

## Step 5: Wire Up Dependencies

### 5.1 Update `src/main.rs`

```rust
use crate::infrastructure::repositories::HardcodedFeedSuggestionsRepository;
use crate::domain::feed_suggestions::FeedSuggestionsService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... existing setup

    // Create feed suggestions repository and service
    let feed_suggestions_repo = Arc::new(HardcodedFeedSuggestionsRepository::new());
    let feed_suggestions_service = Arc::new(FeedSuggestionsService::new(feed_suggestions_repo));

    // Create router with all services
    let app = create_routes(
        feed_suggestions_service,
        // ... other services
    );

    // ... start server
}
```

---

## Step 6: Add E2E Tests

### 6.1 Create test file

```bash
touch tests/e2e/test_feed_suggestions.rs
```

### 6.2 Implement tests

```rust
use crate::helpers::{create_test_context, TestContext};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn it_should_get_all_categories() {
    let ctx = create_test_context().await;
    let token = ctx.create_user_and_login("test@example.com", "password123").await;

    let response = ctx.client
        .get("/api/feed-suggestions/categories")
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    let categories = body["categories"].as_array().unwrap();

    assert_eq!(categories.len(), 20, "Should have exactly 20 categories");
}

#[tokio::test]
#[serial]
async fn it_should_get_suggestions_by_category() {
    let ctx = create_test_context().await;
    let token = ctx.create_user_and_login("test@example.com", "password123").await;

    let response = ctx.client
        .get("/api/feed-suggestions?category_ids=technology-programming")
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    let suggestions = body["suggestions"].as_array().unwrap();

    assert_eq!(suggestions.len(), 4, "Should have exactly 4 suggestions for technology");
}

#[tokio::test]
#[serial]
async fn it_should_deduplicate_multi_category_results() {
    let ctx = create_test_context().await;
    let token = ctx.create_user_and_login("test@example.com", "password123").await;

    let response = ctx.client
        .get("/api/feed-suggestions?category_ids=technology-programming,science-research")
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    let suggestions = body["suggestions"].as_array().unwrap();

    // Should have 8 suggestions (4 from each category, assuming no overlap)
    assert_eq!(suggestions.len(), 8);

    // Verify no duplicate URLs
    let urls: Vec<String> = suggestions
        .iter()
        .map(|s| s["url"].as_str().unwrap().to_string())
        .collect();
    let unique_urls: std::collections::HashSet<_> = urls.iter().collect();
    assert_eq!(urls.len(), unique_urls.len(), "Should have no duplicate URLs");
}
```

---

## Step 7: Run Tests

```bash
# Run all tests
cargo test

# Run only feed suggestions tests
cargo test --test test_feed_suggestions

# Run with output
cargo test --test test_feed_suggestions -- --nocapture
```

---

## Step 8: Manual Testing

### 8.1 Start the server

```bash
cargo run
```

### 8.2 Test with curl

```bash
# 1. Login and get token
TOKEN=$(curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password"}' \
  | jq -r '.access_token')

# 2. Get all categories
curl -X GET http://localhost:8080/api/feed-suggestions/categories \
  -H "Authorization: Bearer $TOKEN" | jq

# 3. Get suggestions for one category
curl -X GET "http://localhost:8080/api/feed-suggestions?category_ids=technology-programming" \
  -H "Authorization: Bearer $TOKEN" | jq

# 4. Get suggestions for multiple categories
curl -X GET "http://localhost:8080/api/feed-suggestions?category_ids=technology-programming,news-current-affairs" \
  -H "Authorization: Bearer $TOKEN" | jq
```

---

## Verification Checklist

- [ ] All 20 categories returned by `/categories` endpoint
- [ ] Each category returns exactly 4 suggestions
- [ ] Multi-category queries return deduplicated results
- [ ] Invalid category IDs are filtered silently (check logs)
- [ ] Endpoints require JWT authentication (401 without token)
- [ ] Response times under 500ms
- [ ] E2E tests pass
- [ ] Code follows domain-driven architecture (controllers → services → repositories)

---

## Common Issues & Solutions

### Issue: LazyLock not found

**Solution**: Ensure Rust version 1.70+. For older versions, use `lazy_static` crate:

```toml
# Cargo.toml
[dependencies]
lazy_static = "1.4"
```

```rust
use lazy_static::lazy_static;

lazy_static! {
    static ref CATEGORIES: Vec<Category> = vec![...];
}
```

### Issue: Tests fail with authentication error

**Solution**: Ensure test helper creates user before attempting login. Check `TestContext` implementation.

### Issue: Repository not implementing trait

**Solution**: Verify trait is `Send + Sync`. Add bounds to trait definition:

```rust
pub trait FeedSuggestionsRepository: Send + Sync {
    // ...
}
```

---

## Next Steps

After implementation:
1. Run `/speckit.tasks` to generate task breakdown
2. Implement tasks in priority order (P1 → P2 → P3)
3. Add remaining 80 feed suggestions to repository (see user-provided list)
4. Test with mobile app integration
5. Monitor response times and user adoption metrics
