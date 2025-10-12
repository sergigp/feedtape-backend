use super::{Category, FeedSuggestion, FeedSuggestionsRepository};
use std::sync::Arc;

pub struct FeedSuggestionsService {
    repository: Arc<dyn FeedSuggestionsRepository>,
}

impl FeedSuggestionsService {
    pub fn new(repository: Arc<dyn FeedSuggestionsRepository>) -> Self {
        Self { repository }
    }

    /// Returns all available categories for display in UI
    pub fn get_categories(&self) -> Vec<Category> {
        self.repository.get_all_categories()
    }

    /// Returns feed suggestions filtered by categories
    /// Returns empty Vec if category_ids is empty
    pub fn get_suggestions(&self, category_ids: Vec<String>) -> Vec<FeedSuggestion> {
        if category_ids.is_empty() {
            tracing::info!("get_suggestions called with empty category_ids");
            return Vec::new();
        }

        tracing::info!(
            category_ids = ?category_ids,
            "Fetching suggestions for categories"
        );

        self.repository.get_suggestions_by_categories(&category_ids)
    }
}
