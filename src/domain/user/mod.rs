pub mod dto;
pub mod error;
pub mod model;
pub mod service;
pub mod voice_mapping;

pub use error::UserServiceError;
pub use model::{SubscriptionStatus, SubscriptionTier, User, UserSettings};
pub use service::{UserService, UserServiceApi};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response for GET /api/me
#[derive(Debug, Serialize, Deserialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub settings: UserSettingsDto,
    pub subscription: SubscriptionDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSettingsDto {
    pub voice: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionDto {
    pub tier: String,
    pub status: String,
    pub usage: UsageDto,
    pub limits: LimitsDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageDto {
    pub minutes_used_today: f32,
    pub minutes_limit: i32,
    pub characters_used_today: i32,
    pub characters_limit: i32,
    pub resets_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitsDto {
    pub max_feeds: i32,
}

/// Request for PATCH /api/me
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMeRequest {
    pub settings: Option<UpdateSettingsDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSettingsDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}
