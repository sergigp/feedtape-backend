# Data Model: Feed Suggestions by Category

**Feature**: Feed Suggestions by Category
**Phase**: 1 (Design)
**Date**: 2025-10-11

## Overview

This document defines the domain entities, value objects, and repository interfaces for the feed suggestions feature. The data model supports 20 categories with 4 curated feed suggestions each.

## Domain Entities

### Category

Represents a content category for organizing feed suggestions.

**Properties**:
- `id: String` - Unique slug identifier (e.g., "technology-programming")
- `name: String` - Display name with emoji (e.g., "💻 Technology & Programming")
- `description: String` - Brief description of category content

**Validation Rules**:
- `id` must be lowercase with hyphens only (slug format)
- `id` must be unique across all categories
- `name` must not be empty
- `description` must be 50-200 characters

**Invariants**:
- Categories are immutable once defined
- Category IDs never change (safe for client caching)

**Example**:
```rust
Category {
    id: "technology-programming".to_string(),
    name: "💻 Technology & Programming".to_string(),
    description: "Latest in tech, programming languages, frameworks, and software development".to_string(),
}
```

---

### FeedSuggestion

Represents a curated RSS feed recommendation within a category.

**Properties**:
- `id: String` - Unique slug identifier (e.g., "techcrunch")
- `title: String` - Feed display title (e.g., "TechCrunch")
- `description: String` - Brief description of feed content
- `url: String` - RSS feed URL (validated, working)
- `category_id: String` - Foreign reference to parent Category

**Validation Rules**:
- `id` must be lowercase with hyphens only (slug format)
- `url` must be valid HTTP/HTTPS URL
- `url` must be unique across all suggestions (same feed can't appear twice)
- `category_id` must reference an existing Category
- `title` must not be empty
- `description` must be 50-300 characters

**Invariants**:
- Feed suggestions are immutable
- Each category has exactly 4 suggestions (validated at compile time in hardcoded data)
- Feed URLs are pre-validated and working

**Example**:
```rust
FeedSuggestion {
    id: "techcrunch".to_string(),
    title: "TechCrunch".to_string(),
    description: "Breaking technology news, analysis, and opinions from Silicon Valley and beyond".to_string(),
    url: "https://techcrunch.com/feed/".to_string(),
    category_id: "technology-programming".to_string(),
}
```

---

## Repository Interface

### FeedSuggestionsRepository Trait

Abstract interface for accessing feed suggestions data. Implementation details (hardcoded, database, API) are hidden from domain layer.

**Methods**:

#### `get_all_categories() -> Vec<Category>`

Returns all available categories.

**Behavior**:
- Returns all 20 categories
- Order: Alphabetical by category name
- No pagination (small, fixed dataset)

**Performance**: O(1) - returns pre-allocated Vec

---

#### `get_suggestions_by_categories(category_ids: &[String]) -> Vec<FeedSuggestion>`

Returns feed suggestions filtered by one or more category IDs.

**Parameters**:
- `category_ids`: Slice of category ID strings to filter by

**Behavior**:
- If `category_ids` is empty: returns empty Vec
- If `category_ids` contains invalid IDs: logs warning, ignores invalid IDs
- If multiple categories requested: deduplicates by feed URL
- Returns suggestions matching ANY of the provided category IDs (OR logic)

**Deduplication**: Uses feed URL as unique key. If same feed appears in multiple requested categories, includes only first occurrence.

**Performance**: O(n) where n = total suggestions (~80)

**Example**:
```rust
// Request suggestions for Technology and Science
let suggestions = repo.get_suggestions_by_categories(&[
    "technology-programming".to_string(),
    "science-research".to_string()
]);
// Returns: 8 suggestions (4 from each category, assuming no overlap)
```

---

## Domain Service

### FeedSuggestionsService

Orchestrates business logic for feed suggestions feature.

**Dependencies**:
- `repository: Arc<dyn FeedSuggestionsRepository>` - Injected via constructor

**Methods**:

#### `get_categories() -> Vec<Category>`

Returns all available categories for display in UI.

**Business Rules**:
- No authentication required at service level (enforced by controller)
- Returns complete category list (no filtering)

---

#### `get_suggestions(category_ids: Vec<String>) -> Result<Vec<FeedSuggestion>, FeedSuggestionsError>`

Returns feed suggestions filtered by categories.

**Business Rules**:
- Empty `category_ids` returns empty result (not an error)
- Invalid category IDs are filtered out silently with warning log
- Deduplicates results by feed URL
- Returns suggestions in deterministic order

**Error Cases**:
- None (filtering invalid IDs is not an error)

---

## Error Types

### FeedSuggestionsError

Domain-specific error type.

**Variants**:

Currently no error variants needed (invalid category IDs filtered silently). Enum created for future extensibility:

```rust
#[derive(Debug, thiserror::Error)]
pub enum FeedSuggestionsError {
    // Future: #[error("Rate limit exceeded")]
    // RateLimitExceeded,
}
```

---

## Data Relationships

```
Category (1) ─────< (N) FeedSuggestion
    │                      │
    │                      │
    └──────────────────────┘
         category_id
```

**Cardinality**:
- 1 Category → exactly 4 FeedSuggestions (enforced in hardcoded data)
- 1 FeedSuggestion → exactly 1 Category
- Total: 20 Categories, 80 FeedSuggestions

**No Many-to-Many**: Each feed belongs to exactly one category. If a feed is conceptually relevant to multiple categories (e.g., TechCrunch in both Technology and Business), it should be duplicated with different suggestion IDs.

---

## Complete Category List

1. news-current-affairs
2. technology-programming
3. science-research
4. business-finance
5. design-creativity
6. gaming-entertainment
7. health-fitness
8. food-cooking
9. travel-adventure
10. books-literature
11. movies-tv
12. music-podcasts
13. sports
14. environment-sustainability
15. politics-policy
16. personal-development
17. lifestyle-home
18. automotive
19. fashion-beauty
20. education-learning

---

## Implementation Notes

### Hardcoded Data Structure

```rust
// src/infrastructure/repositories/feed_suggestions_repository.rs

use std::sync::LazyLock;
use std::collections::{HashMap, HashSet};

static CATEGORIES: LazyLock<Vec<Category>> = LazyLock::new(|| {
    vec![
        Category {
            id: "news-current-affairs".to_string(),
            name: "📰 News & Current Affairs".to_string(),
            description: "Stay informed with breaking news and in-depth analysis".to_string(),
        },
        // ... 19 more categories
    ]
});

static FEED_SUGGESTIONS: LazyLock<Vec<FeedSuggestion>> = LazyLock::new(|| {
    vec![
        // News & Current Affairs (4 feeds)
        FeedSuggestion {
            id: "bbc-news".to_string(),
            title: "BBC News".to_string(),
            description: "Breaking news, analysis and features from the BBC".to_string(),
            url: "https://feeds.bbci.co.uk/news/rss.xml".to_string(),
            category_id: "news-current-affairs".to_string(),
        },
        // ... 79 more suggestions (4 per category × 20 categories)
    ]
});
```

### Validation at Compile Time

Add debug assertion in repository constructor to verify data integrity:

```rust
impl HardcodedFeedSuggestionsRepository {
    pub fn new() -> Self {
        // Verify each category has exactly 4 suggestions
        debug_assert_eq!(CATEGORIES.len(), 20, "Must have exactly 20 categories");
        debug_assert_eq!(FEED_SUGGESTIONS.len(), 80, "Must have exactly 80 suggestions");

        let mut counts = HashMap::new();
        for suggestion in FEED_SUGGESTIONS.iter() {
            *counts.entry(&suggestion.category_id).or_insert(0) += 1;
        }

        for category in CATEGORIES.iter() {
            debug_assert_eq!(
                counts.get(&category.id),
                Some(&4),
                "Category {} must have exactly 4 suggestions",
                category.id
            );
        }

        Self
    }
}
```

---

## Future Considerations

### Database Migration Path

When migrating from hardcoded to database:

1. Create `categories` and `feed_suggestions` tables
2. Add migration script with INSERT statements for current data
3. Implement `DatabaseFeedSuggestionsRepository` struct
4. Implement `FeedSuggestionsRepository` trait for database version
5. Swap implementation in `main.rs` dependency injection
6. Domain and controller layers unchanged

**Tables Schema (future)**:
```sql
CREATE TABLE feed_suggestion_categories (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description VARCHAR(300) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE feed_suggestions (
    id VARCHAR(50) PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    description VARCHAR(500) NOT NULL,
    url VARCHAR(500) NOT NULL UNIQUE,
    category_id VARCHAR(50) NOT NULL REFERENCES feed_suggestion_categories(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_feed_suggestions_category ON feed_suggestions(category_id);
```

### Personalization (future)

To add personalized suggestions:
- Add user preferences table (user_id → favorite_category_ids)
- Extend service method: `get_personalized_suggestions(user_id)`
- Repository remains unchanged (filter in service layer)
