use crate::{
    error::{AppError, AppResult},
    infrastructure::repositories::{FeedRepository, UserRepository},
};
use crate::domain::user::SubscriptionTier;
use uuid::Uuid;
use std::sync::Arc;
use crate::domain::feed::{CreateFeedRequest, FeedResponse, UpdateFeedRequest};

pub struct FeedService {
    feed_repo: Arc<FeedRepository>,
    user_repo: Arc<UserRepository>,
}

impl FeedService {
    pub fn new(
        feed_repo: Arc<FeedRepository>,
        user_repo: Arc<UserRepository>,
    ) -> Self {
        Self {
            feed_repo,
            user_repo,
        }
    }

    /// Get all feeds for a user
    pub async fn get_user_feeds(&self, user_id: Uuid) -> AppResult<Vec<FeedResponse>> {
        let feeds = self.feed_repo.find_by_user(user_id).await?;
        Ok(feeds.into_iter().map(FeedResponse::from).collect())
    }

    /// Create a new feed for a user
    pub async fn create_feed(
        &self,
        user_id: Uuid,
        request: CreateFeedRequest,
    ) -> AppResult<()> {
        // Get user to check subscription tier
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Validate URL format
        if !Self::is_valid_url(&request.url) {
            return Err(AppError::BadRequest("Invalid URL format".to_string()));
        }

        // Check if feed already exists for this user
        if self.feed_repo.exists_for_user(user_id, &request.url).await? {
            return Err(AppError::Conflict(
                "Feed URL already exists".to_string(),
            ));
        }

        // Check feed limit based on subscription tier
        let feed_count = self.feed_repo.count_by_user(user_id).await?;
        let max_feeds = match user.subscription_tier {
            SubscriptionTier::Free => 3,
            SubscriptionTier::Pro => 999,
        };

        if feed_count >= max_feeds {
            return Err(AppError::PaymentRequired(format!(
                "Free tier allows maximum {} feeds. Upgrade to Pro for unlimited feeds.",
                max_feeds
            )));
        }

        // Create the feed with client-provided ID
        self.feed_repo.create(
            request.id,
            user_id,
            &request.url,
            &request.title,
        )
        .await?;

        Ok(())
    }

    /// Update a feed
    pub async fn update_feed(
        &self,
        user_id: Uuid,
        feed_id: Uuid,
        request: UpdateFeedRequest,
    ) -> AppResult<()> {
        // Get the feed and verify ownership
        let feed = self.feed_repo.find_by_id(feed_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Feed not found".to_string()))?;

        if feed.user_id != user_id {
            return Err(AppError::NotFound("Feed not found".to_string()));
        }

        // Update the feed
        self.feed_repo.update_title(feed_id, request.title.as_deref()).await?;

        Ok(())
    }

    /// Delete a feed
    pub async fn delete_feed(&self, user_id: Uuid, feed_id: Uuid) -> AppResult<()> {
        // Get the feed and verify ownership
        let feed = self.feed_repo.find_by_id(feed_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Feed not found".to_string()))?;

        if feed.user_id != user_id {
            return Err(AppError::NotFound("Feed not found".to_string()));
        }

        // Delete the feed
        self.feed_repo.delete(feed_id).await?;

        Ok(())
    }

    /// Basic URL validation
    fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}
