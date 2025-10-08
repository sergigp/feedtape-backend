use crate::{
    error::{AppError, AppResult},
    infrastructure::repositories::{UsageRepository, UserRepository},
};
use super::{detect_language, get_voice_for_language, TtsRequest};
use crate::domain::user::{SubscriptionTier, User};
use aws_sdk_polly::{
    types::{Engine, OutputFormat, VoiceId},
    Client as PollyClient,
};
use uuid::Uuid;
use std::sync::Arc;

const CHARACTERS_PER_MINUTE: f32 = 1000.0;

pub struct TtsService {
    user_repo: Arc<UserRepository>,
    usage_repo: Arc<UsageRepository>,
    polly_client: Arc<PollyClient>,
}

impl TtsService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        usage_repo: Arc<UsageRepository>,
        polly_client: Arc<PollyClient>,
    ) -> Self {
        Self {
            user_repo,
            usage_repo,
            polly_client,
        }
    }

    /// Synthesize text to speech
    pub async fn synthesize(
        &self,
        user_id: Uuid,
        request: TtsRequest,
    ) -> AppResult<(Vec<u8>, String, i32, f32)> {
        // Validate text length
        let char_count = request.text.len() as i32;
        if char_count > 10000 {
            return Err(AppError::PayloadTooLarge(
                "Text must be 10,000 characters or less".to_string(),
            ));
        }

        if char_count == 0 {
            return Err(AppError::BadRequest("Text cannot be empty".to_string()));
        }

        // Get user
        let user = self.user_repo.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Check usage limits
        self.check_usage_limits(&user, char_count).await?;

        // Determine language
        let language = request
            .language
            .as_deref()
            .filter(|l| *l != "auto")
            .map(String::from)
            .unwrap_or_else(|| detect_language(&request.text));

        // Determine quality based on subscription tier
        // Pro users get neural voices, Free users get standard voices
        let quality = match user.subscription_tier {
            SubscriptionTier::Pro => "neural",
            SubscriptionTier::Free => "standard",
        };

        // Select voice based on language and quality
        let voice = get_voice_for_language(&language, quality);

        // Synthesize with Polly
        let audio_data = self.call_polly(&request.text, voice, quality).await?;

        // Calculate minutes (1000 chars = 1 minute)
        let minutes = char_count as f32 / CHARACTERS_PER_MINUTE;

        // Track usage
        self.usage_repo.increment_usage(user_id, char_count).await?;

        Ok((audio_data, language, char_count, minutes))
    }

    /// Check if user has enough quota for this request
    async fn check_usage_limits(&self, user: &User, char_count: i32) -> AppResult<()> {
        // Get today's usage
        let usage = self.usage_repo.get_today_usage(user.id).await?;
        let characters_used_today = usage.map(|u| u.characters_used).unwrap_or(0);

        // Determine character limit
        let character_limit = match user.subscription_tier {
            SubscriptionTier::Free => {
                // Check if trial expired
                if user.is_trial_expired() {
                    return Err(AppError::PaymentRequired(
                        "Free trial expired. Please upgrade to Pro to continue.".to_string(),
                    ));
                }
                20000 // 20 minutes/day = 20,000 characters
            }
            SubscriptionTier::Pro => 200000, // 200 minutes/day = 200,000 characters
        };

        // Check if adding this request would exceed the limit
        if characters_used_today + char_count > character_limit {
            return Err(AppError::PaymentRequired(format!(
                "Daily character limit exceeded. Used: {}, Limit: {}, Request: {}",
                characters_used_today, character_limit, char_count
            )));
        }

        Ok(())
    }

    /// Call AWS Polly to synthesize speech
    async fn call_polly(
        &self,
        text: &str,
        voice: &str,
        quality: &str,
    ) -> AppResult<Vec<u8>> {
        // Parse voice ID
        let voice_id = VoiceId::from(voice);

        // Determine engine (neural or standard)
        let engine = if quality == "neural" {
            Engine::Neural
        } else {
            Engine::Standard
        };

        // Call Polly
        let result = self.polly_client
            .synthesize_speech()
            .text(text)
            .voice_id(voice_id)
            .output_format(OutputFormat::Mp3)
            .engine(engine)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("AWS Polly error: {}", e)))?;

        // Get audio stream
        let audio_stream = result
            .audio_stream
            .collect()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to read audio stream: {}", e)))?;

        Ok(audio_stream.into_bytes().to_vec())
    }
}
