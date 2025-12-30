pub mod error;
pub mod model;
pub mod service;

pub use error::FeedServiceError;
pub use model::Feed;
pub use service::{FeedService, FeedServiceApi};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response for feed endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct FeedResponse {
    pub id: Uuid,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_read_at: Option<DateTime<Utc>>,
}

/// Request to create a new feed
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedRequest {
    pub id: Uuid,
    pub url: String,
    pub title: String,
}

/// Request to update last read timestamp
#[derive(Debug, Deserialize)]
pub struct UpdateLastReadRequest {
    pub last_read_at: DateTime<Utc>,
}

impl From<Feed> for FeedResponse {
    fn from(feed: Feed) -> Self {
        Self {
            id: feed.id,
            url: feed.url,
            title: feed.title,
            created_at: feed.created_at,
            last_read_at: feed.last_read_at,
        }
    }
}
