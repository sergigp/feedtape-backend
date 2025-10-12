# Research: Feed Suggestions by Category

**Feature**: Feed Suggestions by Category
**Phase**: 0 (Research & Architecture)
**Date**: 2025-10-11

## Overview

This document captures research findings and architectural decisions for implementing a feed suggestions system with hardcoded data using the repository pattern.

## Research Areas

### 1. Repository Pattern for In-Memory Data

**Decision**: Implement trait-based repository with hardcoded data in infrastructure layer

**Rationale**:
- Abstracts data source from domain logic
- Enables future migration to database or external API without changing service layer
- Maintains consistency with existing codebase architecture (following constitution principle II)
- Allows easy testing with mock implementations

**Alternatives Considered**:
1. **Direct hardcoded HashMap in service**
   - Rejected: Violates repository pattern, makes testing harder, couples domain to implementation
2. **Database storage from day one**
   - Rejected: Over-engineering for static data, adds unnecessary complexity, slows iteration
3. **External configuration file (JSON/YAML)**
   - Rejected: Requires file I/O, error handling for parsing, doesn't leverage Rust's type safety at compile time

**Implementation Approach**:
```rust
// Domain layer: src/domain/feed_suggestions/mod.rs
pub trait FeedSuggestionsRepository {
    fn get_all_categories(&self) -> Vec<Category>;
    fn get_suggestions_by_categories(&self, category_ids: &[String]) -> Vec<FeedSuggestion>;
}

// Infrastructure layer: src/infrastructure/repositories/feed_suggestions_repository.rs
pub struct HardcodedFeedSuggestionsRepository {
    categories: Vec<Category>,
    suggestions: Vec<FeedSuggestion>,
}

impl FeedSuggestionsRepository for HardcodedFeedSuggestionsRepository {
    // Implementation with lazy_static or once_cell for static data
}
```

---

### 2. Category Identifier Strategy

**Decision**: Use slug-style string identifiers (e.g., "technology-programming", "news-current-affairs")

**Rationale**:
- Human-readable and URL-friendly
- Easy to use in REST API query parameters
- No database auto-increment dependency
- Self-documenting in logs and API calls
- Case-insensitive comparison possible

**Alternatives Considered**:
1. **Numeric IDs (1, 2, 3...)**
   - Rejected: Not meaningful, requires ID-to-name lookup everywhere, harder to debug
2. **UUID v4**
   - Rejected: Overkill for 20 static categories, not human-readable
3. **Uppercase constants (TECHNOLOGY, NEWS)**
   - Rejected: Not URL-friendly, less flexible for display names

**Examples**:
- "news-current-affairs"
- "technology-programming"
- "science-research"
- "business-finance"

---

### 3. Data Deduplication Strategy

**Decision**: Use `HashSet<String>` with feed URL as unique key when merging multiple categories

**Rationale**:
- Feed URL is naturally unique across the system
- O(1) lookup for duplicate detection
- Rust's `HashSet` is efficient for small sets (80 total feeds)
- Preserves first occurrence order (can convert to Vec after dedup)

**Alternatives Considered**:
1. **Check if feed already in result Vec**
   - Rejected: O(n²) complexity, slower for multiple categories
2. **Use feed title as unique key**
   - Rejected: Titles might not be globally unique (e.g., "Home" or "Blog")
3. **Database DISTINCT query**
   - Rejected: Not using database for this feature

**Implementation**:
```rust
pub fn get_suggestions_by_categories(&self, category_ids: &[String]) -> Vec<FeedSuggestion> {
    let mut seen_urls = HashSet::new();
    let mut results = Vec::new();

    for suggestion in &self.suggestions {
        if category_ids.contains(&suggestion.category_id) && seen_urls.insert(&suggestion.url) {
            results.push(suggestion.clone());
        }
    }

    results
}
```

---

### 4. Rust Pattern for Static Hardcoded Data

**Decision**: Use `lazy_static!` or `once_cell::sync::Lazy` for compile-time static data

**Rationale**:
- Data initialized once at first access
- Zero runtime overhead after initialization
- Thread-safe by default
- Better than const because allows complex types (Vec, HashMap)

**Alternatives Considered**:
1. **Load from const arrays**
   - Rejected: Requires array syntax, less flexible for complex structures
2. **Build in repository constructor**
   - Rejected: Rebuilds data on every instantiation, wastes memory
3. **Load from JSON file at startup**
   - Rejected: Adds file I/O dependency, error handling, loses compile-time type safety

**Implementation Choice**: `once_cell::sync::Lazy` (preferred, already in Rust std as of 1.70)

```rust
use std::sync::LazyLock;

static CATEGORIES: LazyLock<Vec<Category>> = LazyLock::new(|| {
    vec![
        Category {
            id: "news-current-affairs".to_string(),
            name: "📰 News & Current Affairs".to_string(),
            description: "Stay informed with breaking news...".to_string(),
        },
        // ... 19 more
    ]
});
```

---

### 5. API Response Format & Pagination

**Decision**: Return all results without pagination for MVP

**Rationale**:
- Maximum 80 feeds total across all categories
- Maximum 4 feeds per category
- Small payload size (~50KB JSON for all feeds with descriptions)
- Simplifies client implementation
- Aligns with success criteria (<500ms response time)

**Alternatives Considered**:
1. **Pagination with limit/offset**
   - Rejected: Over-engineering for static list of 4-80 items
2. **Cursor-based pagination**
   - Rejected: No ordering/sorting requirements, data doesn't change
3. **GraphQL with field selection**
   - Rejected: REST API is project standard, adds new dependency

**Future Consideration**: If categories expand to 50+ with 10+ feeds each, implement pagination.

---

### 6. Error Handling for Invalid Category IDs

**Decision**: Filter out invalid IDs silently, return suggestions for valid IDs only

**Rationale**:
- Resilient to client errors (typos, outdated cache)
- Allows partial results (valid categories still returned)
- Logs warning for debugging
- Better UX than failing entire request

**Alternatives Considered**:
1. **Return 400 Bad Request for any invalid ID**
   - Rejected: Too strict, breaks if client has cached old category IDs
2. **Return 404 Not Found**
   - Rejected: Some categories might be valid, denies partial results
3. **Return error details in response**
   - Rejected: Clutters successful responses, client can validate upfront

**Implementation**:
```rust
pub fn get_suggestions_by_categories(&self, category_ids: &[String]) -> Vec<FeedSuggestion> {
    let valid_ids: HashSet<_> = self.categories.iter().map(|c| &c.id).collect();

    for id in category_ids {
        if !valid_ids.contains(id) {
            tracing::warn!(category_id = %id, "Invalid category ID requested");
        }
    }

    // Filter and return only valid suggestions
}
```

---

### 7. Authentication Requirements

**Decision**: Both endpoints require JWT authentication (reuse existing middleware)

**Rationale**:
- Prevents abuse from unauthenticated users
- Aligns with spec requirement FR-008
- Consistent with other API endpoints
- Enables user analytics (which categories are popular)

**Implementation**: Apply existing `auth_middleware` to routes

---

### 8. Response Schema Structure

**Decision**: Nested structure with category metadata included in each feed suggestion

**Rationale**:
- Client gets complete context without additional lookups
- Self-documenting responses
- Slightly larger payload but avoids N+1 client queries

**Response Format**:
```json
{
  "categories": [
    {
      "id": "technology-programming",
      "name": "💻 Technology & Programming",
      "description": "Latest in tech, programming, and innovation"
    }
  ],
  "suggestions": [
    {
      "id": "techcrunch",
      "title": "TechCrunch",
      "description": "Latest technology news and insights",
      "url": "https://techcrunch.com/feed/",
      "category": {
        "id": "technology-programming",
        "name": "💻 Technology & Programming"
      }
    }
  ]
}
```

**Alternatives Considered**:
1. **Separate category IDs only in suggestions** (client does lookup)
   - Rejected: Requires client to build lookup map, more complex
2. **Group suggestions by category** (nested array)
   - Rejected: Harder to filter, doesn't match query model (multi-category)

---

## Technology Stack Confirmation

**Language**: Rust 1.70+
**Framework**: Axum 0.7
**Serialization**: serde + serde_json
**Static Data**: `once_cell` or `lazy_static`
**Testing**: cargo test with `TestContext` from existing e2e helpers

**No New Dependencies Required**: Feature uses existing project dependencies.

---

## Performance Considerations

**Expected Performance**:
- Category list endpoint: <10ms (no I/O, returns static Vec)
- Suggestions endpoint: <50ms (in-memory filtering, deduplication)
- Memory footprint: ~100KB for all 80 feed suggestions

**Benchmarks**: Not needed for MVP (well under 500ms requirement with in-memory data).

---

## Summary

All technical unknowns resolved. Feature can proceed to Phase 1 (Design & Contracts) with:
- Repository pattern using trait + hardcoded implementation
- Slug-based category identifiers
- HashSet for deduplication
- LazyLock/lazy_static for static data
- No pagination (return all results)
- Silent filtering of invalid category IDs with logging
- JWT authentication on all endpoints
- Nested response format with category metadata
