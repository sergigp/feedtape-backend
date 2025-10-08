use crate::{
    error::{AppError, AppResult},
    infrastructure::repositories::{UsageRepository, UserRepository},
};
use super::{LimitsDto, MeResponse, SubscriptionDto, UpdateSettingsDto, UsageDto, User, UserSettingsDto};
use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;

/// Voice ID mapping - maps voice names to voice IDs
fn get_voice_id(voice_name: &str) -> String {
    let voice_map: HashMap<&str, &str> = [
        ("Lucia", "voice_lucia_es"),
        ("Sergio", "voice_sergio_es"),
        ("Conchita", "voice_conchita_es"),
        ("Matthew", "voice_matthew_en"),
        ("Joanna", "voice_joanna_en"),
        ("Amy", "voice_amy_en"),
        ("Celine", "voice_celine_fr"),
        ("Mathieu", "voice_mathieu_fr"),
        ("Hans", "voice_hans_de"),
        ("Marlene", "voice_marlene_de"),
        ("Ricardo", "voice_ricardo_pt"),
        ("Ines", "voice_ines_pt"),
        ("Carla", "voice_carla_it"),
        ("Giorgio", "voice_giorgio_it"),
    ]
    .iter()
    .cloned()
    .collect();

    voice_map
        .get(voice_name)
        .unwrap_or(&"voice_lucia_es")
        .to_string()
}

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
    ) -> AppResult<()> {
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
        if let Some(language) = updates.language {
            if !["es", "en", "fr", "de", "pt", "it"].contains(&language.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Invalid language: {}",
                    language
                )));
            }
            settings["language"] = json!(language);
        }

        // Update in database
        self.user_repo.update_settings(user_id, settings).await?;

        Ok(())
    }

    /// Build MeResponse from user and usage data
    fn build_me_response(
        user: &User,
        usage: Option<&crate::infrastructure::repositories::UsageRecord>,
    ) -> AppResult<MeResponse> {
        // Parse settings
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

        // Calculate limits based on tier
        let (characters_limit, minutes_limit, max_feeds) = match user.subscription_tier {
            crate::domain::user::SubscriptionTier::Free => (20000, 20, 3),
            crate::domain::user::SubscriptionTier::Pro => (200000, 200, 999),
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
