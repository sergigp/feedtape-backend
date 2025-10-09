use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    Extension, Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        tts::{TtsService, TtsServiceApi},
        user::{UserService, UserServiceApi},
        shared::usage_dto::{DailyUsage, UsageLimits, UsageResponse, UsageStats},
    },
    error::{AppError, AppResult},
    infrastructure::{
        auth::AuthUser,
        repositories::UsageRepository,
    },
};
use chrono::{Duration, Utc};

/// Request for POST /api/tts/synthesize
#[derive(Debug, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub link: String,
}

pub struct TtsController {
    tts_service: Arc<TtsService>,
    user_service: Arc<UserService>,
    usage_repo: Arc<UsageRepository>,
}

impl TtsController {
    pub fn new(
        tts_service: Arc<TtsService>,
        user_service: Arc<UserService>,
        usage_repo: Arc<UsageRepository>,
    ) -> Self {
        Self {
            tts_service,
            user_service,
            usage_repo,
        }
    }

    /// POST /api/tts/synthesize - Convert text to speech
    pub async fn synthesize(
        State(controller): State<Arc<TtsController>>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<TtsRequest>,
    ) -> AppResult<(StatusCode, HeaderMap, Body)> {
        // Validate input
        let char_count = request.text.len() as i32;

        if char_count == 0 {
            return Err(AppError::BadRequest("Text cannot be empty".to_string()));
        }

        if char_count > 10000 {
            return Err(AppError::PayloadTooLarge(
                "Text must be 10,000 characters or less".to_string(),
            ));
        }

        // Synthesize speech using service
        let result = controller.tts_service
            .synthesize(auth_user.user_id, request.text, request.link)
            .await
            .map_err(|e| AppError::from(e))?;

        // Calculate duration in seconds (approximate)
        let duration_seconds = (result.duration_minutes * 60.0) as u64;

        // Get remaining usage
        let usage = controller.usage_repo.get_today_usage(auth_user.user_id)
            .await?;
        let characters_used = usage.map(|u| u.characters_used).unwrap_or(0);
        let character_limit = 20000; // This should come from user's tier, simplified for now

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "audio/mpeg".parse().unwrap());
        headers.insert(
            "X-Duration-Seconds",
            duration_seconds.to_string().parse().unwrap(),
        );
        headers.insert(
            "X-Character-Count",
            result.char_count.to_string().parse().unwrap(),
        );
        headers.insert(
            "X-Language-Detected",
            result.language_detected.to_string().parse().unwrap(),
        );
        headers.insert(
            "X-Usage-Remaining",
            (character_limit - characters_used).to_string().parse().unwrap(),
        );

        Ok((StatusCode::OK, headers, Body::from(result.audio_data)))
    }

    /// GET /api/tts/usage - Get usage statistics
    pub async fn get_usage(
        State(controller): State<Arc<TtsController>>,
        Extension(auth_user): Extension<AuthUser>,
    ) -> AppResult<Json<UsageResponse>> {
        // Get user profile to determine limits
        let me_response = controller.user_service.get_user_profile(auth_user.user_id).await?;

        // Get today's usage
        let today_usage = controller.usage_repo.get_today_usage(auth_user.user_id).await?;

        let (characters_used, articles_count) = if let Some(usage) = &today_usage {
            (usage.characters_used, usage.articles_synthesized)
        } else {
            (0, 0)
        };

        // Calculate minutes from characters (1000 chars = 1 minute)
        let minutes_used = characters_used as f32 / 1000.0;

        // Get limits from user profile
        let character_limit = me_response.subscription.usage.characters_limit;
        let minute_limit = me_response.subscription.usage.minutes_limit;

        // Get usage history (last 30 days)
        let history_records = controller.usage_repo.get_usage_history(auth_user.user_id, 30).await?;
        let history: Vec<DailyUsage> = history_records
            .into_iter()
            .map(|r| DailyUsage {
                date: r.date,
                characters: r.characters_used,
                minutes: r.characters_used as f32 / 1000.0, // Calculate minutes from characters
            })
            .collect();

        // Calculate reset time (midnight tonight)
        let now = Utc::now();
        let tomorrow = now + Duration::days(1);
        let resets_at = tomorrow
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        Ok(Json(UsageResponse {
            period: "daily".to_string(),
            usage: UsageStats {
                characters: characters_used,
                minutes: minutes_used,
                requests: articles_count,
            },
            limits: UsageLimits {
                characters: character_limit,
                minutes: minute_limit,
                requests: 999999, // No request limit
            },
            resets_at,
            history: Some(history),
        }))
    }
}
