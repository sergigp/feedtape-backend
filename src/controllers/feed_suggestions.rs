use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    domain::feed_suggestions::{Category, FeedSuggestionsService},
    error::AppResult,
    infrastructure::auth::AuthUser,
};

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct GetSuggestionsQuery {
    #[serde(default)]
    pub category_ids: Option<String>, // Comma-separated
    #[serde(default)]
    pub categories: Option<String>, // Alias for category_ids
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct FeedSuggestionResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct CategoryWithSuggestionsResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub suggestions: Vec<FeedSuggestionResponse>,
}

#[derive(Debug, Serialize)]
pub struct SuggestionsResponse {
    pub categories: Vec<CategoryWithSuggestionsResponse>,
}

pub struct FeedSuggestionsController {
    service: Arc<FeedSuggestionsService>,
}

impl FeedSuggestionsController {
    pub fn new(service: Arc<FeedSuggestionsService>) -> Self {
        Self { service }
    }

    /// GET /api/feed-suggestions - Get categories with their feed suggestions
    /// If category_ids is provided, returns only those categories.
    /// If no category_ids provided, returns all categories.
    pub async fn get_suggestions(
        State(controller): State<Arc<FeedSuggestionsController>>,
        Extension(_auth_user): Extension<AuthUser>,
        Query(query): Query<GetSuggestionsQuery>,
    ) -> AppResult<Json<SuggestionsResponse>> {
        // Parse category IDs from query params (support both parameter names)
        let category_ids_filter: Option<Vec<String>> = query
            .category_ids
            .or(query.categories)
            .map(|s| s.split(',').map(|id| id.trim().to_string()).collect());

        let all_categories = controller.service.get_categories();

        // Filter categories if specific IDs were requested
        let categories_to_return: Vec<Category> = if let Some(ref filter_ids) = category_ids_filter
        {
            all_categories
                .into_iter()
                .filter(|cat| filter_ids.contains(&cat.id))
                .collect()
        } else {
            all_categories
        };

        // Build response with nested suggestions for each category
        let mut response_categories: Vec<CategoryWithSuggestionsResponse> = Vec::new();

        for category in categories_to_return {
            // Get suggestions for this specific category
            let suggestions = controller.service.get_suggestions(vec![category.id.clone()]);

            let suggestion_responses: Vec<FeedSuggestionResponse> = suggestions
                .into_iter()
                .map(|s| FeedSuggestionResponse {
                    id: s.id,
                    title: s.title,
                    description: s.description,
                    url: s.url,
                })
                .collect();

            response_categories.push(CategoryWithSuggestionsResponse {
                id: category.id,
                name: category.name,
                description: category.description,
                suggestions: suggestion_responses,
            });
        }

        Ok(Json(SuggestionsResponse {
            categories: response_categories,
        }))
    }
}
