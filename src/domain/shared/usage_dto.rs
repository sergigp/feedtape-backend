use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Response for GET /api/tts/usage
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageResponse {
    pub period: String,
    pub usage: UsageStats,
    pub limits: UsageLimits,
    pub resets_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<DailyUsage>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageStats {
    pub characters: i32,
    pub minutes: f32,
    pub requests: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageLimits {
    pub characters: i32,
    pub minutes: i32,
    pub requests: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: NaiveDate,
    pub characters: i32,
    pub minutes: f32,
}
