use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub oauth_provider: String,
    pub oauth_provider_id: String,
    pub settings: JsonValue,
    pub subscription_tier: SubscriptionTier,
    pub subscription_status: SubscriptionStatus,
    pub subscription_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum SubscriptionTier {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "pro")]
    Pro,
}

impl std::fmt::Display for SubscriptionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionTier::Free => write!(f, "free"),
            SubscriptionTier::Pro => write!(f, "pro"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionStatus::Active => write!(f, "active"),
            SubscriptionStatus::Expired => write!(f, "expired"),
            SubscriptionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// User settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub voice: String,
    pub speed: f32,
    pub language: String,
    pub quality: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            voice: "Lucia".to_string(),
            speed: 1.0,
            language: "auto".to_string(),
            quality: "standard".to_string(),
        }
    }
}

impl User {
    /// Check if user is on free trial (trial = first 7 days from account creation)
    pub fn is_trial(&self) -> bool {
        let days_since_signup = Utc::now()
            .signed_duration_since(self.created_at)
            .num_days();
        self.subscription_tier == SubscriptionTier::Free && days_since_signup < 7
    }

    /// Check if trial has expired
    pub fn is_trial_expired(&self) -> bool {
        let days_since_signup = Utc::now()
            .signed_duration_since(self.created_at)
            .num_days();
        self.subscription_tier == SubscriptionTier::Free && days_since_signup >= 7
    }
}
