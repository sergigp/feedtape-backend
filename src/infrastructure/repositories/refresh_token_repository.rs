use crate::infrastructure::db::DbPool;
use crate::{error::AppResult};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;
use std::sync::Arc;

pub struct RefreshTokenRepository {
    pool: Arc<DbPool>,
}

impl RefreshTokenRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Create a new refresh token
    pub async fn create(
        &self,
        user_id: Uuid,
        token: &str,
        expiration_days: i64,
    ) -> AppResult<()> {
        let pool = self.pool.as_ref();
        let id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::days(expiration_days);

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token, expires_at, created_at, revoked)
            VALUES ($1, $2, $3, $4, $5, false)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Find a valid (non-revoked, non-expired) refresh token
    pub async fn find_valid(
        &self,
        token: &str,
    ) -> AppResult<Option<(Uuid, DateTime<Utc>)>> {
        let pool = self.pool.as_ref();
        let result = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
            r#"
            SELECT user_id, expires_at
            FROM refresh_tokens
            WHERE token = $1
              AND NOT revoked
              AND expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Check if a refresh token exists and get its status
    pub async fn check_token_status(
        &self,
        token: &str,
    ) -> AppResult<Option<(bool, bool)>> {
        let pool = self.pool.as_ref();
        let result = sqlx::query_as::<_, (bool, DateTime<Utc>)>(
            r#"
            SELECT revoked, expires_at
            FROM refresh_tokens
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        if let Some((revoked, expires_at)) = result {
            let is_expired = expires_at <= Utc::now();
            Ok(Some((revoked, is_expired)))
        } else {
            Ok(None)
        }
    }

    /// Revoke a refresh token
    pub async fn revoke(&self, token: &str) -> AppResult<()> {
        let pool = self.pool.as_ref();
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE
            WHERE token = $1
            "#,
        )
        .bind(token)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Revoke all refresh tokens for a user
    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> AppResult<()> {
        let pool = self.pool.as_ref();
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE
            WHERE user_id = $1 AND NOT revoked
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete expired refresh tokens (cleanup)
    pub async fn delete_expired(&self) -> AppResult<u64> {
        let pool = self.pool.as_ref();
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
