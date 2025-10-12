use super::error::TtsServiceError;
use super::language::LanguageCode;
use crate::domain::user::{SubscriptionTier, User};
use crate::infrastructure::repositories::{TtsRepository, UsageRepository, UserRepository};
use async_trait::async_trait;
use html2text::from_read;
use lingua::{LanguageDetector, LanguageDetectorBuilder};
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

const CHARACTERS_PER_MINUTE: f32 = 1000.0;

#[derive(Debug, Clone)]
pub struct TtsSynthesisResult {
    pub audio_data: Vec<u8>,
    pub language_detected: LanguageCode,
    pub char_count: i32,
    pub duration_minutes: f32,
}

pub struct TtsService {
    user_repo: Arc<UserRepository>,
    usage_repo: Arc<UsageRepository>,
    tts_repo: Arc<dyn TtsRepository>,
    language_detector: LanguageDetector,
    cache: Option<Cache<String, TtsSynthesisResult>>,
}

impl TtsService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        usage_repo: Arc<UsageRepository>,
        tts_repo: Arc<dyn TtsRepository>,
        cache_enabled: bool,
    ) -> Self {
        // Create language detector with the languages we support in Cargo.toml
        let language_detector = LanguageDetectorBuilder::from_all_languages().build();

        // Initialize cache if enabled
        let cache = if cache_enabled {
            Some(
                Cache::builder()
                    .max_capacity(100)
                    .time_to_idle(Duration::from_secs(30 * 60)) // 30 minutes, refreshes on access
                    .build(),
            )
        } else {
            None
        };

        Self {
            user_repo,
            usage_repo,
            tts_repo,
            language_detector,
            cache,
        }
    }
}

#[async_trait]
pub trait TtsServiceApi: Send + Sync {
    /// Synthesize text to speech for a given user
    ///
    /// This operation:
    /// - Validates user exists and has quota
    /// - Calls AWS Polly for synthesis (English, neural voice)
    /// - Tracks usage
    ///
    /// Returns audio data along with metadata (language, char count, duration)
    async fn synthesize(
        &self,
        user_id: Uuid,
        text: String,
        link: String,
    ) -> Result<TtsSynthesisResult, TtsServiceError>;
}

#[async_trait]
impl TtsServiceApi for TtsService {
    async fn synthesize(
        &self,
        user_id: Uuid,
        text: String,
        link: String,
    ) -> Result<TtsSynthesisResult, TtsServiceError> {
        // Log analytics data
        tracing::info!(
            user_id = %user_id,
            link = %link,
            text_length = text.len(),
            "TTS synthesis request"
        );

        // Check cache first (if enabled)
        if let Some(cache) = &self.cache {
            if let Some(cached_result) = cache.get(&link).await {
                tracing::info!(
                    link = %link,
                    cached_audio_size = cached_result.audio_data.len(),
                    cached_char_count = cached_result.char_count,
                    cached_language = %cached_result.language_detected,
                    "TTS cache hit - returning cached audio"
                );
                return Ok(cached_result);
            }
        }

        // 1. Clean the text (remove HTML, URLs, normalize whitespace)
        let cleaned_text = self.clean_text(&text);
        let char_count = cleaned_text.len() as i32;

        tracing::info!(
            original_length = text.len(),
            cleaned_length = cleaned_text.len(),
            "Text cleaned"
        );

        // 2. Detect language from cleaned text
        let detected_language = self.detect_language(&cleaned_text);

        tracing::info!(
            link = %link,
            language_detected = %detected_language,
            "Language detected for TTS synthesis"
        );

        // 3. Find user
        let user = self.find_user(user_id).await?;

        // 4. Guard usage limits
        self.guard_usage(&user, char_count).await?;

        // 5. Call TTS repository to synthesize (repository handles splitting/merging)
        let audio_data = self
            .tts_repo
            .synthesize(&cleaned_text, detected_language)
            .await
            .map_err(|e| TtsServiceError::Dependency(e))?;

        // 6. Track usage
        self.track_usage(user_id, char_count).await?;

        // 7. Calculate duration and create result
        let duration_minutes = char_count as f32 / CHARACTERS_PER_MINUTE;

        let result = TtsSynthesisResult {
            audio_data,
            language_detected: detected_language,
            char_count,
            duration_minutes,
        };

        // 8. Cache the result if caching is enabled
        if let Some(cache) = &self.cache {
            cache.insert(link.clone(), result.clone()).await;
            tracing::info!(
                link = %link,
                audio_size = result.audio_data.len(),
                "TTS result cached"
            );
        }

        Ok(result)
    }
}

impl TtsService {
    async fn find_user(&self, user_id: Uuid) -> Result<User, TtsServiceError> {
        self.user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| TtsServiceError::Dependency(e.to_string()))?
            .ok_or_else(|| TtsServiceError::Invalid("User not found".to_string()))
    }

    async fn guard_usage(&self, user: &User, char_count: i32) -> Result<(), TtsServiceError> {
        let usage = self
            .usage_repo
            .get_today_usage(user.id)
            .await
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

    async fn track_usage(&self, user_id: Uuid, char_count: i32) -> Result<(), TtsServiceError> {
        self.usage_repo
            .increment_usage(user_id, char_count)
            .await
            .map_err(|e| TtsServiceError::Dependency(e.to_string()))
    }

    /// Detect language from text
    fn detect_language(&self, text: &str) -> LanguageCode {
        match self.language_detector.detect_language_of(text) {
            Some(language) => {
                // Convert lingua Language enum to LanguageCode
                LanguageCode::from_lingua(language)
            }
            None => {
                tracing::warn!("Could not detect language, falling back to English");
                LanguageCode::English
            }
        }
    }

    /// Clean text by removing HTML tags and normalizing whitespace
    fn clean_text(&self, text: &str) -> String {
        // Convert HTML to plain text
        let plain_text = from_read(text.as_bytes(), usize::MAX);

        // Remove URLs (both http and https)
        let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();
        let without_urls = url_pattern.replace_all(&plain_text, "");

        // Normalize whitespace (replace multiple spaces/newlines with single space)
        let whitespace_pattern = regex::Regex::new(r"\s+").unwrap();
        let normalized = whitespace_pattern.replace_all(&without_urls, " ");

        normalized.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingua::Language;

    // Test helper function that mirrors the service method
    fn clean_text_test(text: &str) -> String {
        let plain_text = from_read(text.as_bytes(), usize::MAX);
        let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();
        let without_urls = url_pattern.replace_all(&plain_text, "");
        let whitespace_pattern = regex::Regex::new(r"\s+").unwrap();
        let normalized = whitespace_pattern.replace_all(&without_urls, " ");
        normalized.trim().to_string()
    }

    #[test]
    fn test_clean_text_removes_html() {
        let input = "<p>Hello <strong>world</strong>!</p>";
        let result = clean_text_test(input);
        // html2text converts <strong> to markdown bold **
        // The important thing is HTML tags are removed
        assert!(!result.contains("<"));
        assert!(!result.contains(">"));
        assert!(result.contains("Hello"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_clean_text_removes_urls() {
        let input = "Check this out https://example.com and http://test.com";
        let result = clean_text_test(input);
        assert!(!result.contains("https://"));
        assert!(!result.contains("http://"));
        assert!(result.contains("Check this out"));
    }

    #[test]
    fn test_clean_text_normalizes_whitespace() {
        let input = "Too    many     spaces\n\nand\n\nnewlines";
        let result = clean_text_test(input);
        assert!(!result.contains("  ")); // No double spaces
        assert_eq!(result, "Too many spaces and newlines");
    }

    #[test]
    fn test_clean_text_handles_complex_html() {
        let input = r#"
            <html>
                <body>
                    <h1>Title</h1>
                    <p>Paragraph with <a href="https://example.com">link</a>.</p>
                    <div>Another section https://test.com here.</div>
                </body>
            </html>
        "#;
        let result = clean_text_test(input);
        assert!(!result.contains("<"));
        assert!(!result.contains(">"));
        assert!(!result.contains("https://"));
        assert!(result.contains("Title"));
        assert!(result.contains("Paragraph"));
    }

    #[test]
    fn test_detect_language_english() {
        let detector = LanguageDetectorBuilder::from_all_languages().build();
        let text = "This is a test in English. The quick brown fox jumps over the lazy dog.";
        let language = detector.detect_language_of(text);
        assert_eq!(language, Some(Language::English));
    }

    #[test]
    fn test_detect_language_spanish() {
        let detector = LanguageDetectorBuilder::from_all_languages().build();
        let text =
            "Esto es una prueba en español. El rápido zorro marrón salta sobre el perro perezoso.";
        let language = detector.detect_language_of(text);
        assert_eq!(language, Some(Language::Spanish));
    }

    #[test]
    fn test_detect_language_french() {
        let detector = LanguageDetectorBuilder::from_all_languages().build();
        let text = "Ceci est un test en français. Le rapide renard brun saute par-dessus le chien paresseux.";
        let language = detector.detect_language_of(text);
        assert_eq!(language, Some(Language::French));
    }

    #[test]
    fn test_detect_language_german() {
        let detector = LanguageDetectorBuilder::from_all_languages().build();
        let text = "Dies ist ein Test auf Deutsch. Der schnelle braune Fuchs springt über den faulen Hund.";
        let language = detector.detect_language_of(text);
        assert_eq!(language, Some(Language::German));
    }

    #[test]
    fn test_detect_language_italian() {
        let detector = LanguageDetectorBuilder::from_all_languages().build();
        let text =
            "Questo è un test in italiano. La volpe marrone veloce salta sopra il cane pigro.";
        let language = detector.detect_language_of(text);
        assert_eq!(language, Some(Language::Italian));
    }

    #[test]
    fn test_detect_language_portuguese() {
        let detector = LanguageDetectorBuilder::from_all_languages().build();
        let text =
            "Este é um teste em português. A rápida raposa marrom salta sobre o cão preguiçoso.";
        let language = detector.detect_language_of(text);
        assert_eq!(language, Some(Language::Portuguese));
    }
}
