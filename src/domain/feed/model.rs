use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Feed {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
}

impl Feed {
    /// Update the last_read_at timestamp
    /// Returns error if the timestamp is in the future
    pub fn update_last_read_at(&mut self, last_read_at: DateTime<Utc>) -> Result<(), String> {
        if last_read_at > Utc::now() {
            return Err("last_read_at cannot be in the future".to_string());
        }
        self.last_read_at = Some(last_read_at);
        Ok(())
    }
}
