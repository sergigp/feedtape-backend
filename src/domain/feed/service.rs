use super::error::FeedServiceError;
use crate::domain::feed::{CreateFeedRequest, Feed, FeedResponse, UpdateFeedRequest};
use crate::domain::user::{SubscriptionTier, User};
use crate::infrastructure::repositories::{FeedRepository, UserRepository};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

const MAX_FEEDS_FREE: i64 = 3;
const MAX_FEEDS_PRO: i64 = 999;

pub struct FeedService {
    feed_repo: Arc<FeedRepository>,
    user_repo: Arc<UserRepository>,
}

impl FeedService {
    pub fn new(feed_repo: Arc<FeedRepository>, user_repo: Arc<UserRepository>) -> Self {
        Self {
            feed_repo,
            user_repo,
        }
    }
}

#[async_trait]
pub trait FeedServiceApi: Send + Sync {
    async fn get_user_feeds(&self, user_id: Uuid) -> Result<Vec<FeedResponse>, FeedServiceError>;

    async fn create_feed(
        &self,
        user_id: Uuid,
        request: CreateFeedRequest,
    ) -> Result<(), FeedServiceError>;

    async fn update_feed(
        &self,
        user_id: Uuid,
        feed_id: Uuid,
        request: UpdateFeedRequest,
    ) -> Result<(), FeedServiceError>;

    async fn delete_feed(&self, user_id: Uuid, feed_id: Uuid) -> Result<(), FeedServiceError>;

    async fn update_last_read_at(
        &self,
        user_id: Uuid,
        feed_id: Uuid,
        last_read_at: DateTime<Utc>,
    ) -> Result<(), FeedServiceError>;
}

#[async_trait]
impl FeedServiceApi for FeedService {
    async fn get_user_feeds(&self, user_id: Uuid) -> Result<Vec<FeedResponse>, FeedServiceError> {
        let feeds = self
            .feed_repo
            .find_by_user(user_id)
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?;
        Ok(feeds.into_iter().map(FeedResponse::from).collect())
    }

    async fn create_feed(
        &self,
        user_id: Uuid,
        request: CreateFeedRequest,
    ) -> Result<(), FeedServiceError> {
        let user = self.find_user(user_id).await?;

        self.validate_url(&request.url)?;

        if self
            .feed_repo
            .exists_for_user(user_id, &request.url)
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?
        {
            return Err(FeedServiceError::Conflict);
        }

        self.check_feed_limit(user_id, user.subscription_tier)
            .await?;

        self.feed_repo
            .create(request.id, user_id, &request.url, &request.title)
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?;

        Ok(())
    }

    async fn update_feed(
        &self,
        user_id: Uuid,
        feed_id: Uuid,
        request: UpdateFeedRequest,
    ) -> Result<(), FeedServiceError> {
        self.verify_feed_ownership(feed_id, user_id).await?;

        self.feed_repo
            .update_title(feed_id, request.title.as_deref())
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?;

        Ok(())
    }

    async fn delete_feed(&self, user_id: Uuid, feed_id: Uuid) -> Result<(), FeedServiceError> {
        self.verify_feed_ownership(feed_id, user_id).await?;

        self.feed_repo
            .delete(feed_id)
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?;

        Ok(())
    }

    async fn update_last_read_at(
        &self,
        user_id: Uuid,
        feed_id: Uuid,
        last_read_at: DateTime<Utc>,
    ) -> Result<(), FeedServiceError> {
        self.verify_feed_ownership(feed_id, user_id).await?;

        if last_read_at > Utc::now() {
            return Err(FeedServiceError::Invalid(
                "last_read_at cannot be in the future".to_string(),
            ));
        }

        self.feed_repo
            .update_last_read_at(feed_id, last_read_at)
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?;

        Ok(())
    }
}

impl FeedService {
    async fn find_user(&self, user_id: Uuid) -> Result<User, FeedServiceError> {
        self.user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?
            .ok_or_else(|| FeedServiceError::Invalid("User not found".to_string()))
    }

    fn validate_url(&self, url: &str) -> Result<(), FeedServiceError> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(FeedServiceError::Invalid("Invalid URL format".to_string()));
        }
        Ok(())
    }

    async fn check_feed_limit(
        &self,
        user_id: Uuid,
        tier: SubscriptionTier,
    ) -> Result<(), FeedServiceError> {
        let feed_count = self
            .feed_repo
            .count_by_user(user_id)
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?;

        let max_feeds = match tier {
            SubscriptionTier::Free => MAX_FEEDS_FREE,
            SubscriptionTier::Pro => MAX_FEEDS_PRO,
        };

        if feed_count >= max_feeds {
            return Err(FeedServiceError::PaymentRequired(format!(
                "Free tier allows maximum {} feeds. Upgrade to Pro for unlimited feeds.",
                max_feeds
            )));
        }

        Ok(())
    }

    async fn verify_feed_ownership(
        &self,
        feed_id: Uuid,
        user_id: Uuid,
    ) -> Result<Feed, FeedServiceError> {
        let feed = self
            .feed_repo
            .find_by_id(feed_id)
            .await
            .map_err(|e| FeedServiceError::Dependency(e.to_string()))?
            .ok_or(FeedServiceError::NotFound)?;

        if feed.user_id != user_id {
            return Err(FeedServiceError::NotFound);
        }

        Ok(feed)
    }
}
