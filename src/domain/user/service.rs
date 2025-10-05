use crate::{
    error::{AppError, AppResult},
    infrastructure::repositories::{UsageRepository, UserRepository},
};
use super::{dto::*, User};
use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;
use std::sync::Arc;

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

    /// Get user profile with subscription and usage info
    pub async fn get_user_profile(&self, user_id: Uuid) -> AppResult<MeResponse> {
        // Get user
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Get today's usage
        let usage = self.usage_repo.get_today_usage(user_id).await?;

        let response = Self::build_me_response(&user, usage.as_ref())?;

        Ok(response)
    }

    /// Update user settings
    pub async fn update_user_settings(
        &self,
        user_id: Uuid,
        updates: UpdateSettingsDto,
    ) -> AppResult<MeResponse> {
        // Get current user
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Parse current settings
        let mut settings: serde_json::Value = user.settings.clone();

        // Apply updates
        if let Some(voice) = updates.voice {
            settings["voice"] = json!(voice);
        }
        if let Some(speed) = updates.speed {
            if speed < 0.5 || speed > 2.0 {
                return Err(AppError::BadRequest(
                    "Speed must be between 0.5 and 2.0".to_string(),
                ));
            }
            settings["speed"] = json!(speed);
        }
        if let Some(language) = updates.language {
            if !["auto", "es", "en", "fr", "de", "pt", "it"].contains(&language.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Invalid language: {}",
                    language
                )));
            }
            settings["language"] = json!(language);
        }
        if let Some(quality) = updates.quality {
            if !["standard", "neural"].contains(&quality.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Invalid quality: {}",
                    quality
                )));
            }
            settings["quality"] = json!(quality);
        }

        // Update in database
        let updated_user = self.user_repo.update_settings(user_id, settings).await?;

        // Get usage
        let usage = self.usage_repo.get_today_usage(user_id).await?;

        let response = Self::build_me_response(&updated_user, usage.as_ref())?;

        Ok(response)
    }

    /// Build MeResponse from user and usage data
    fn build_me_response(
        user: &User,
        usage: Option<&crate::infrastructure::repositories::UsageRecord>,
    ) -> AppResult<MeResponse> {
        // Parse settings
        let settings_json = &user.settings;
        let voice = settings_json
            .get("voice")
            .and_then(|v| v.as_str())
            .unwrap_or("Lucia")
            .to_string();
        let speed = settings_json
            .get("speed")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        let language = settings_json
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("auto")
            .to_string();
        let quality = settings_json
            .get("quality")
            .and_then(|v| v.as_str())
            .unwrap_or("standard")
            .to_string();

        // Calculate limits based on tier
        let (characters_limit, minutes_limit, max_feeds, voice_quality) = match user
            .subscription_tier
        {
            crate::domain::user::SubscriptionTier::Free => (20000, 20, 3, "standard"),
            crate::domain::user::SubscriptionTier::Pro => (200000, 200, 999, "neural"),
        };

        // Get usage stats
        let characters_used_today = usage.map(|u| u.characters_used).unwrap_or(0);
        // Calculate minutes from characters (1000 chars = 1 minute)
        let minutes_used_today = characters_used_today as f32 / 1000.0;

        // Calculate reset time (midnight tonight)
        let now = Utc::now();
        let tomorrow = now + Duration::days(1);
        let resets_at = tomorrow
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        Ok(MeResponse {
            id: user.id,
            email: user.email.clone(),
            settings: UserSettingsDto {
                voice,
                speed,
                language,
                quality,
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
                    voice_quality: voice_quality.to_string(),
                },
                expires_at: user.subscription_expires_at,
                store: None, // iOS only - no need to track store in DB anymore
            },
        })
    }
}
