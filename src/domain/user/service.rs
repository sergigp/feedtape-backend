use crate::infrastructure::repositories::{UsageRecord, UsageRepository, UserRepository};
use super::{LimitsDto, MeResponse, SubscriptionDto, UpdateSettingsDto, UsageDto, User, UserSettingsDto};
use super::error::UserServiceError;
use super::voice_mapping::get_voice_id;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;

const CHARACTERS_PER_MINUTE: f32 = 1000.0;
const FREE_TIER_CHARACTERS: i32 = 20000;
const FREE_TIER_MINUTES: i32 = 20;
const FREE_TIER_MAX_FEEDS: i32 = 3;
const PRO_TIER_CHARACTERS: i32 = 200000;
const PRO_TIER_MINUTES: i32 = 200;
const PRO_TIER_MAX_FEEDS: i32 = 999;
const SUPPORTED_LANGUAGES: &[&str] = &["es", "en", "fr", "de", "pt", "it"];

pub struct UserService {
    user_repo: Arc<UserRepository>,
    usage_repo: Arc<UsageRepository>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        usage_repo: Arc<UsageRepository>,
    ) -> Self {
        Self {
            user_repo,
            usage_repo,
        }
    }
}

#[async_trait]
pub trait UserServiceApi: Send + Sync {
    async fn get_user_profile(&self, user_id: Uuid) -> Result<MeResponse, UserServiceError>;

    async fn update_user_settings(
        &self,
        user_id: Uuid,
        updates: UpdateSettingsDto,
    ) -> Result<(), UserServiceError>;
}

#[async_trait]
impl UserServiceApi for UserService {
    async fn get_user_profile(&self, user_id: Uuid) -> Result<MeResponse, UserServiceError> {
        let user = self.find_user(user_id).await?;
        let usage = self.get_today_usage(user_id).await?;

        let response = Self::build_me_response(&user, usage.as_ref())?;

        Ok(response)
    }

    async fn update_user_settings(
        &self,
        user_id: Uuid,
        updates: UpdateSettingsDto,
    ) -> Result<(), UserServiceError> {
        let user = self.find_user(user_id).await?;

        let mut settings: serde_json::Value = user.settings.clone();

        if let Some(voice) = updates.voice {
            settings["voice"] = json!(voice);
        }
        if let Some(language) = &updates.language {
            self.validate_language(language)?;
            settings["language"] = json!(language);
        }

        self.user_repo
            .update_settings(user_id, settings)
            .await
            .map_err(|e| UserServiceError::Dependency(e.to_string()))?;

        Ok(())
    }
}

impl UserService {
    async fn find_user(&self, user_id: Uuid) -> Result<User, UserServiceError> {
        self.user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| UserServiceError::Dependency(e.to_string()))?
            .ok_or(UserServiceError::NotFound)
    }

    async fn get_today_usage(&self, user_id: Uuid) -> Result<Option<UsageRecord>, UserServiceError> {
        self.usage_repo
            .get_today_usage(user_id)
            .await
            .map_err(|e| UserServiceError::Dependency(e.to_string()))
    }

    fn validate_language(&self, language: &str) -> Result<(), UserServiceError> {
        if !SUPPORTED_LANGUAGES.contains(&language) {
            return Err(UserServiceError::Invalid(format!(
                "Invalid language: {}",
                language
            )));
        }
        Ok(())
    }

    fn calculate_limits(tier: crate::domain::user::SubscriptionTier) -> (i32, i32, i32) {
        match tier {
            crate::domain::user::SubscriptionTier::Free => {
                (FREE_TIER_CHARACTERS, FREE_TIER_MINUTES, FREE_TIER_MAX_FEEDS)
            }
            crate::domain::user::SubscriptionTier::Pro => {
                (PRO_TIER_CHARACTERS, PRO_TIER_MINUTES, PRO_TIER_MAX_FEEDS)
            }
        }
    }

    fn calculate_reset_time() -> DateTime<Utc> {
        let now = Utc::now();
        let tomorrow = now + Duration::days(1);
        tomorrow
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
    }

    fn build_me_response(
        user: &User,
        usage: Option<&UsageRecord>,
    ) -> Result<MeResponse, UserServiceError> {
        let settings_json = &user.settings;
        let voice_name = settings_json
            .get("voice")
            .and_then(|v| v.as_str())
            .unwrap_or("Lucia");
        let voice_id = get_voice_id(voice_name);
        let language = settings_json
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("en")
            .to_string();

        let (characters_limit, minutes_limit, max_feeds) = Self::calculate_limits(user.subscription_tier.clone());

        let characters_used_today = usage.map(|u| u.characters_used).unwrap_or(0);
        let minutes_used_today = characters_used_today as f32 / CHARACTERS_PER_MINUTE;

        let resets_at = Self::calculate_reset_time();

        Ok(MeResponse {
            id: user.id,
            settings: UserSettingsDto {
                voice: voice_id,
                language,
            },
            subscription: SubscriptionDto {
                tier: user.subscription_tier.to_string(),
                status: user.subscription_status.to_string(),
                usage: UsageDto {
                    minutes_used_today,
                    minutes_limit,
                    characters_used_today,
                    characters_limit,
                    resets_at,
                },
                limits: LimitsDto {
                    max_feeds,
                },
            },
        })
    }
}
