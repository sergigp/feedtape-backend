use crate::infrastructure::db::DbPool;
use crate::{domain::user::User, error::AppResult};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub struct UserRepository {
    pool: Arc<DbPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Find user by ID
    pub async fn find_by_id(&self, user_id: Uuid) -> AppResult<Option<User>> {
        let pool = self.pool.as_ref();
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let pool = self.pool.as_ref();
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    /// Find user by OAuth provider and provider ID
    pub async fn find_by_oauth(
        &self,
        provider: &str,
        provider_id: &str,
    ) -> AppResult<Option<User>> {
        let pool = self.pool.as_ref();
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE oauth_provider = $1 AND oauth_provider_id = $2",
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create(&self, email: &str, provider: &str, provider_id: &str) -> AppResult<User> {
        let pool = self.pool.as_ref();
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let default_settings = json!({
            "voice": "Lucia",
            "speed": 1.0,
            "language": "auto",
            "quality": "standard"
        });

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, oauth_provider, oauth_provider_id, settings, subscription_tier, subscription_status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'free', 'active', $6, $6)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(email)
        .bind(provider)
        .bind(provider_id)
        .bind(default_settings)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Update user settings
    pub async fn update_settings(
        &self,
        user_id: Uuid,
        settings: serde_json::Value,
    ) -> AppResult<User> {
        let pool = self.pool.as_ref();
        let now = chrono::Utc::now();

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET settings = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(settings)
        .bind(now)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}
