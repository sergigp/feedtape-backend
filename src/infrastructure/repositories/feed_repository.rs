use crate::infrastructure::db::DbPool;
use crate::{
    domain::feed::Feed,
    error::{AppError, AppResult},
};
use std::sync::Arc;
use uuid::Uuid;

pub struct FeedRepository {
    pool: Arc<DbPool>,
}

impl FeedRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Get all feeds for a user
    pub async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Feed>> {
        let pool = self.pool.as_ref();
        let feeds = sqlx::query_as::<_, Feed>(
            r#"
            SELECT id, user_id, url, title, created_at
            FROM feeds
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(feeds)
    }

    /// Get a feed by ID
    pub async fn find_by_id(&self, feed_id: Uuid) -> AppResult<Option<Feed>> {
        let pool = self.pool.as_ref();
        let feed = sqlx::query_as::<_, Feed>(
            r#"
            SELECT id, user_id, url, title, created_at
            FROM feeds
            WHERE id = $1
            "#,
        )
        .bind(feed_id)
        .fetch_optional(pool)
        .await?;

        Ok(feed)
    }

    /// Check if a user already has a feed with this URL
    pub async fn exists_for_user(&self, user_id: Uuid, url: &str) -> AppResult<bool> {
        let pool = self.pool.as_ref();
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM feeds
                WHERE user_id = $1 AND url = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(url)
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }

    /// Count feeds for a user
    pub async fn count_by_user(&self, user_id: Uuid) -> AppResult<i64> {
        let pool = self.pool.as_ref();
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM feeds
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Create a new feed with client-provided ID
    pub async fn create(&self, id: Uuid, user_id: Uuid, url: &str, title: &str) -> AppResult<()> {
        let pool = self.pool.as_ref();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO feeds (id, user_id, url, title, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(url)
        .bind(title)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("Feed URL already exists".to_string());
                }
            }
            AppError::Database(e)
        })?;

        Ok(())
    }

    /// Update a feed (title)
    pub async fn update(&self, feed: &Feed) -> AppResult<()> {
        let pool = self.pool.as_ref();
        sqlx::query(
            r#"
            UPDATE feeds
            SET title = $1
            WHERE id = $2
            "#,
        )
        .bind(&feed.title)
        .bind(feed.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete a feed
    pub async fn delete(&self, feed_id: Uuid) -> AppResult<bool> {
        let pool = self.pool.as_ref();
        let result = sqlx::query(
            r#"
            DELETE FROM feeds
            WHERE id = $1
            "#,
        )
        .bind(feed_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
