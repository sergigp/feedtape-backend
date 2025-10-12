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
