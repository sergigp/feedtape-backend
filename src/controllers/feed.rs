use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::feed::dto::{CreateFeedRequest, FeedResponse, UpdateFeedRequest},
    error::AppResult,
    infrastructure::auth::AuthUser,
    domain::feed::FeedService,
};

pub struct FeedController {
    feed_service: Arc<FeedService>,
}

impl FeedController {
    pub fn new(feed_service: Arc<FeedService>) -> Self {
        Self { feed_service }
    }

    /// GET /api/feeds - List user's feeds
    pub async fn list_feeds(
        State(controller): State<Arc<FeedController>>,
        Extension(auth_user): Extension<AuthUser>,
    ) -> AppResult<Json<Vec<FeedResponse>>> {
        let feeds = controller.feed_service.get_user_feeds(auth_user.user_id).await?;
        Ok(Json(feeds))
    }

    /// POST /api/feeds - Create new feed
    pub async fn create_feed(
        State(controller): State<Arc<FeedController>>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<CreateFeedRequest>,
    ) -> AppResult<(StatusCode, Json<FeedResponse>)> {
        let feed = controller.feed_service.create_feed(auth_user.user_id, request).await?;
        Ok((StatusCode::CREATED, Json(feed)))
    }

    /// PUT /api/feeds/{feedId} - Update feed
    pub async fn update_feed(
        State(controller): State<Arc<FeedController>>,
        Extension(auth_user): Extension<AuthUser>,
        Path(feed_id): Path<Uuid>,
        Json(request): Json<UpdateFeedRequest>,
    ) -> AppResult<Json<FeedResponse>> {
        let feed = controller.feed_service.update_feed(auth_user.user_id, feed_id, request).await?;
        Ok(Json(feed))
    }

    /// DELETE /api/feeds/{feedId} - Delete feed
    pub async fn delete_feed(
        State(controller): State<Arc<FeedController>>,
        Extension(auth_user): Extension<AuthUser>,
        Path(feed_id): Path<Uuid>,
    ) -> AppResult<StatusCode> {
        controller.feed_service.delete_feed(auth_user.user_id, feed_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
