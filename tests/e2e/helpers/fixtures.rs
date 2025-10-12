use anyhow::Result;
use chrono::{DateTime, Utc};
use feedtape_backend::domain::{
    feed::model::Feed,
    user::model::{SubscriptionStatus, SubscriptionTier, User, UserSettings},
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TestFixtures {
    pool: PgPool,
}

impl TestFixtures {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, email: &str) -> Result<User> {
        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            oauth_provider: "google".to_string(),
            oauth_provider_id: format!("provider_{}", Uuid::new_v4()),
            settings: serde_json::to_value(&UserSettings::default())?,
            subscription_tier: SubscriptionTier::Free,
            subscription_status: SubscriptionStatus::Active,
            subscription_expires_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, oauth_provider, oauth_provider_id,
                settings, subscription_tier, subscription_status,
                subscription_expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.oauth_provider)
        .bind(&user.oauth_provider_id)
        .bind(&user.settings)
        .bind(&user.subscription_tier.to_string())
        .bind(&user.subscription_status.to_string())
        .bind(&user.subscription_expires_at)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create_pro_user(&self, email: &str) -> Result<User> {
        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            oauth_provider: "google".to_string(),
            oauth_provider_id: format!("provider_{}", Uuid::new_v4()),
            settings: serde_json::to_value(&UserSettings::default())?,
            subscription_tier: SubscriptionTier::Pro,
            subscription_status: SubscriptionStatus::Active,
            subscription_expires_at: Some(Utc::now() + chrono::Duration::days(30)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, oauth_provider, oauth_provider_id,
                settings, subscription_tier, subscription_status,
                subscription_expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.oauth_provider)
        .bind(&user.oauth_provider_id)
        .bind(&user.settings)
        .bind(&user.subscription_tier.to_string())
        .bind(&user.subscription_status.to_string())
        .bind(&user.subscription_expires_at)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create_feed(&self, user_id: Uuid, url: &str, title: Option<&str>) -> Result<Feed> {
        let feed = Feed {
            id: Uuid::new_v4(),
            user_id,
            url: url.to_string(),
            title: title.map(|s| s.to_string()),
            created_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO feeds (id, user_id, url, title, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&feed.id)
        .bind(&feed.user_id)
        .bind(&feed.url)
        .bind(&feed.title)
        .bind(&feed.created_at)
        .execute(&self.pool)
        .await?;

        Ok(feed)
    }

    pub async fn create_multiple_feeds(&self, user_id: Uuid, count: usize) -> Result<Vec<Feed>> {
        let mut feeds = Vec::new();
        for i in 0..count {
            let feed = self
                .create_feed(
                    user_id,
                    &format!("https://blog{}.example.com/rss", i),
                    Some(&format!("Blog {}", i)),
                )
                .await?;
            feeds.push(feed);
        }
        Ok(feeds)
    }

    pub async fn create_refresh_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
        revoked: bool,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token, expires_at, created_at, revoked)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(token) // In real app this would be hashed
        .bind(expires_at)
        .bind(Utc::now())
        .bind(revoked)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_tts_usage(&self, user_id: Uuid, characters: i32, articles: i32) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO usage_tracking (id, user_id, characters_used, articles_synthesized, date, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id, date)
            DO UPDATE SET
                characters_used = usage_tracking.characters_used + EXCLUDED.characters_used,
                articles_synthesized = usage_tracking.articles_synthesized + EXCLUDED.articles_synthesized,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(characters)
        .bind(articles)
        .bind(Utc::now().date_naive())
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_feed_count(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM feeds WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    #[allow(dead_code)]
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<
            _,
            (
                Uuid,
                String,
                String,
                String,
                serde_json::Value,
                String,
                String,
                Option<DateTime<Utc>>,
                DateTime<Utc>,
                DateTime<Utc>,
            ),
        >(
            r#"
            SELECT id, email, oauth_provider, oauth_provider_id, settings,
                   subscription_tier, subscription_status, subscription_expires_at,
                   created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((
            id,
            email,
            oauth_provider,
            oauth_provider_id,
            settings,
            subscription_tier,
            subscription_status,
            subscription_expires_at,
            created_at,
            updated_at,
        )) = user
        {
            let tier = match subscription_tier.as_str() {
                "pro" => SubscriptionTier::Pro,
                _ => SubscriptionTier::Free,
            };

            let status = match subscription_status.as_str() {
                "expired" => SubscriptionStatus::Expired,
                "cancelled" => SubscriptionStatus::Cancelled,
                _ => SubscriptionStatus::Active,
            };

            Ok(Some(User {
                id,
                email,
                oauth_provider,
                oauth_provider_id,
                settings,
                subscription_tier: tier,
                subscription_status: status,
                subscription_expires_at,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }
}
