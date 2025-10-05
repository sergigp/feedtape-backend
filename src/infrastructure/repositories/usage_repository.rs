use crate::{error::AppResult};
use chrono::{NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use crate::infrastructure::db::DbPool;
use std::sync::Arc;

#[derive(Debug, FromRow)]
pub struct UsageRecord {
    pub user_id: Uuid,
    pub date: NaiveDate,
    pub characters_used: i32,
    pub articles_synthesized: i32,
}

pub struct UsageRepository {
    pool: Arc<DbPool>,
}

impl UsageRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Get today's usage for a user
    pub async fn get_today_usage(&self, user_id: Uuid) -> AppResult<Option<UsageRecord>> {
        let pool = self.pool.as_ref();
        let today = Utc::now().date_naive();

        let usage = sqlx::query_as::<_, UsageRecord>(
            r#"
            SELECT user_id, date, characters_used, articles_synthesized
            FROM usage_tracking
            WHERE user_id = $1 AND date = $2
            "#,
        )
        .bind(user_id)
        .bind(today)
        .fetch_optional(pool)
        .await?;

        Ok(usage)
    }

    /// Increment usage for today
    pub async fn increment_usage(
        &self,
        user_id: Uuid,
        characters: i32,
    ) -> AppResult<()> {
        let pool = self.pool.as_ref();
        let now = Utc::now();
        let today = now.date_naive();
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO usage_tracking (id, user_id, date, characters_used, articles_synthesized, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 1, $5, $5)
            ON CONFLICT (user_id, date)
            DO UPDATE SET
                characters_used = usage_tracking.characters_used + $4,
                articles_synthesized = usage_tracking.articles_synthesized + 1,
                updated_at = $5
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(today)
        .bind(characters)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get usage history for a user
    pub async fn get_usage_history(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<UsageRecord>> {
        let pool = self.pool.as_ref();
        let records = sqlx::query_as::<_, UsageRecord>(
            r#"
            SELECT user_id, date, characters_used, articles_synthesized
            FROM usage_tracking
            WHERE user_id = $1
            ORDER BY date DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }
}
