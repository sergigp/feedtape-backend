use crate::infrastructure::repositories::{UsageRepository, UserRepository};
use super::{detect_language, get_voice_for_language, TtsRequest};
use super::error::TtsServiceError;
use crate::domain::user::{SubscriptionTier, User};
use aws_sdk_polly::{
    types::{Engine, OutputFormat, VoiceId},
    Client as PollyClient,
};
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;

const CHARACTERS_PER_MINUTE: f32 = 1000.0;

#[derive(Debug, Clone)]
pub struct TtsSynthesisResult {
    pub audio_data: Vec<u8>,
    pub language_detected: String,
    pub char_count: i32,
    pub duration_minutes: f32,
}

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
}

#[async_trait]
pub trait TtsServiceApi: Send + Sync {
    /// Synthesize text to speech for a given user
    ///
    /// This operation:
    /// - Validates user exists and has quota
    /// - Detects language and selects appropriate voice
    /// - Calls AWS Polly for synthesis
    /// - Tracks usage
    ///
    /// Returns audio data along with metadata (language, char count, duration)
    async fn synthesize(
        &self,
        user_id: Uuid,
        request: TtsRequest,
    ) -> Result<TtsSynthesisResult, TtsServiceError>;
}

#[async_trait]
impl TtsServiceApi for TtsService {
    async fn synthesize(
        &self,
        user_id: Uuid,
        request: TtsRequest,
    ) -> Result<TtsSynthesisResult, TtsServiceError> {
        // Log analytics data
        tracing::info!(
            user_id = %user_id,
            link = %request.link,
            text_length = request.text.len(),
            "TTS synthesis request"
        );

        let char_count = request.text.len() as i32;

        // 1. Find user
        let user = self.find_user(user_id).await?;

        // 2. Guard usage limits
        self.guard_usage(&user, char_count).await?;

        // 3. Detect language
        let language = self.detect_language_from_request(&request);

        // 4. Select voice based on user tier and language
        let (voice, quality) = self.select_voice(&user, &language);

        // 5. Call Polly to synthesize
        let audio_data = self.call_polly(&request.text, voice, quality).await?;

        // 6. Track usage
        self.track_usage(user_id, char_count).await?;

        // 7. Calculate duration and return result
        let duration_minutes = char_count as f32 / CHARACTERS_PER_MINUTE;

        Ok(TtsSynthesisResult {
            audio_data,
            language_detected: language,
            char_count,
            duration_minutes,
        })
    }
}

impl TtsService {
    fn detect_language_from_request(&self, request: &TtsRequest) -> String {
        request
            .language
            .as_deref()
            .filter(|l| *l != "auto")
            .map(String::from)
            .unwrap_or_else(|| detect_language(&request.text))
    }

    fn select_voice<'a>(&self, user: &User, language: &str) -> (&'a str, &'a str) {
        let quality = match user.subscription_tier {
            SubscriptionTier::Pro => "neural",
            SubscriptionTier::Free => "standard",
        };
        let voice = get_voice_for_language(language, quality);
        (voice, quality)
    }

    async fn find_user(&self, user_id: Uuid) -> Result<User, TtsServiceError> {
        self.user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| TtsServiceError::Dependency(e.to_string()))?
            .ok_or_else(|| TtsServiceError::Invalid("User not found".to_string()))
    }

    async fn guard_usage(&self, user: &User, char_count: i32) -> Result<(), TtsServiceError> {
        let usage = self.usage_repo.get_today_usage(user.id).await
            .map_err(|e| TtsServiceError::Dependency(e.to_string()))?;
        let characters_used_today = usage.map(|u| u.characters_used).unwrap_or(0);

        // Determine character limit based on tier
        let character_limit = match user.subscription_tier {
            SubscriptionTier::Free => {
                // Check if trial expired
                if user.is_trial_expired() {
                    return Err(TtsServiceError::PaymentRequired(
                        "Free trial expired. Please upgrade to Pro to continue.".to_string(),
                    ));
                }
                20000 // 20 minutes/day = 20,000 characters
            }
            SubscriptionTier::Pro => 200000, // 200 minutes/day = 200,000 characters
        };

        // Check if adding this request would exceed the limit
        if characters_used_today + char_count > character_limit {
            return Err(TtsServiceError::PaymentRequired(format!(
                "Daily character limit exceeded. Used: {}, Limit: {}, Request: {}",
                characters_used_today, character_limit, char_count
            )));
        }

        Ok(())
    }

    async fn call_polly(
        &self,
        text: &str,
        voice: &str,
        quality: &str,
    ) -> Result<Vec<u8>, TtsServiceError> {
        // Parse voice ID
        let voice_id = VoiceId::from(voice);

        // Determine engine (neural or standard)
        let engine = if quality == "neural" {
            Engine::Neural
        } else {
            Engine::Standard
        };

        // Log the full request details for debugging
        tracing::info!(
            voice = %voice,
            voice_id = ?voice_id,
            engine = ?engine,
            output_format = "Mp3",
            text_length = text.len(),
            text_preview = &text[..text.len().min(200)],
            "Calling AWS Polly synthesize_speech"
        );

        // Call Polly
        let result = self.polly_client
            .synthesize_speech()
            .text(text)
            .voice_id(voice_id)
            .output_format(OutputFormat::Mp3)
            .engine(engine.clone())
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    error = ?e,
                    error_display = %e,
                    voice = %voice,
                    engine = ?engine,
                    text_length = text.len(),
                    "AWS Polly synthesize_speech failed"
                );
                TtsServiceError::Dependency(format!("AWS Polly error: {:?}", e))
            })?;

        tracing::debug!("AWS Polly synthesize_speech successful, reading audio stream");

        // Get audio stream
        let audio_stream = result
            .audio_stream
            .collect()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to collect audio stream from Polly response");
                TtsServiceError::Dependency(format!("Failed to read audio stream: {}", e))
            })?;

        let audio_bytes = audio_stream.into_bytes().to_vec();
        tracing::debug!(audio_size = audio_bytes.len(), "Audio stream collected successfully");

        Ok(audio_bytes)
    }

    async fn track_usage(&self, user_id: Uuid, char_count: i32) -> Result<(), TtsServiceError> {
        self.usage_repo
            .increment_usage(user_id, char_count)
            .await
            .map_err(|e| TtsServiceError::Dependency(e.to_string()))
    }
}
