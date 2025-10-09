use crate::infrastructure::repositories::{UsageRepository, UserRepository};
use super::error::TtsServiceError;
use super::language::LanguageCode;
use crate::domain::user::{SubscriptionTier, User};
use aws_sdk_polly::{
    types::{Engine, OutputFormat, VoiceId},
    Client as PollyClient,
};
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;
use html2text::from_read;
use lingua::{LanguageDetector, LanguageDetectorBuilder};

const CHARACTERS_PER_MINUTE: f32 = 1000.0;
const MAX_BATCH_SIZE: usize = 3000;

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
    polly_client: Arc<PollyClient>,
    language_detector: LanguageDetector,
}

impl TtsService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        usage_repo: Arc<UsageRepository>,
        polly_client: Arc<PollyClient>,
    ) -> Self {
        // Create language detector with the languages we support in Cargo.toml
        let language_detector = LanguageDetectorBuilder::from_all_languages().build();

        Self {
            user_repo,
            usage_repo,
            polly_client,
            language_detector,
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

        // 5. Split text into batches
        let batches = self.split_into_batches(&cleaned_text);
        tracing::info!(
            batch_count = batches.len(),
            "Text split into batches"
        );

        // 6. Call Polly for each batch and merge results using the detected language
        let audio_data = self.synthesize_batches(&batches, detected_language).await?;

        // 7. Track usage
        self.track_usage(user_id, char_count).await?;

        // 8. Calculate duration and return result
        let duration_minutes = char_count as f32 / CHARACTERS_PER_MINUTE;

        Ok(TtsSynthesisResult {
            audio_data,
            language_detected: detected_language,
            char_count,
            duration_minutes,
        })
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

    async fn call_polly(&self, text: &str, language_code: LanguageCode) -> Result<Vec<u8>, TtsServiceError> {
        // Select voice based on detected language (always use neural)
        let voice_name = super::language::get_voice_for_language(language_code, "neural");
        let voice_id = VoiceId::from(voice_name);
        let engine = Engine::Neural;

        // Log the full request details for debugging
        tracing::info!(
            language = %language_code,
            voice = voice_name,
            voice_id = ?voice_id,
            engine = ?engine,
            output_format = "Mp3",
            text_length = text.len(),
            text_preview = &text[..text.len().min(200)],
            "Calling AWS Polly synthesize_speech"
        );

        // Clone voice_id for error logging since it will be moved
        let voice_id_for_error = voice_id.clone();

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
                    language = %language_code,
                    voice_id = ?voice_id_for_error,
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

    /// Synthesize multiple text batches and merge the audio results in order
    async fn synthesize_batches(&self, batches: &[String], language_code: LanguageCode) -> Result<Vec<u8>, TtsServiceError> {
        let mut merged_audio = Vec::new();

        for (index, batch) in batches.iter().enumerate() {
            tracing::info!(
                batch_index = index,
                batch_size = batch.len(),
                "Synthesizing batch"
            );

            let audio_data = self.call_polly(batch, language_code).await?;
            merged_audio.extend(audio_data);

            tracing::info!(
                batch_index = index,
                total_audio_size = merged_audio.len(),
                "Batch synthesized and merged"
            );
        }

        Ok(merged_audio)
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

    /// Split text into batches that respect sentence boundaries
    /// Each batch is at most MAX_BATCH_SIZE characters
    fn split_into_batches(&self, text: &str) -> Vec<String> {
        if text.len() <= MAX_BATCH_SIZE {
            return vec![text.to_string()];
        }

        let mut batches = Vec::new();
        let mut current_batch = String::new();

        // Split on sentence-ending punctuation
        let sentence_pattern = regex::Regex::new(r"([.!?]+\s+)").unwrap();
        let mut last_end = 0;

        for mat in sentence_pattern.find_iter(text) {
            let sentence = &text[last_end..mat.end()];

            // If adding this sentence would exceed the limit, save current batch
            if !current_batch.is_empty() && current_batch.len() + sentence.len() > MAX_BATCH_SIZE {
                batches.push(current_batch.trim().to_string());
                current_batch = String::new();
            }

            current_batch.push_str(sentence);
            last_end = mat.end();
        }

        // Handle remaining text after last sentence boundary
        if last_end < text.len() {
            let remaining = &text[last_end..];

            // If we have a current batch and adding remaining would exceed limit
            if !current_batch.is_empty() && current_batch.len() + remaining.len() > MAX_BATCH_SIZE {
                batches.push(current_batch.trim().to_string());
                current_batch = String::new();
            }

            // If remaining text itself is too large, split it by characters
            if remaining.len() > MAX_BATCH_SIZE {
                let chars: Vec<char> = remaining.chars().collect();
                for chunk in chars.chunks(MAX_BATCH_SIZE) {
                    batches.push(chunk.iter().collect());
                }
            } else {
                current_batch.push_str(remaining);
            }
        }

        // Add any remaining batch
        if !current_batch.is_empty() {
            batches.push(current_batch.trim().to_string());
        }

        batches
    }
}

#[cfg(test)]
mod tests {
    use lingua::Language;
    use super::*;

    // Test helper functions that mirror the service methods
    fn clean_text_test(text: &str) -> String {
        let plain_text = from_read(text.as_bytes(), usize::MAX);
        let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();
        let without_urls = url_pattern.replace_all(&plain_text, "");
        let whitespace_pattern = regex::Regex::new(r"\s+").unwrap();
        let normalized = whitespace_pattern.replace_all(&without_urls, " ");
        normalized.trim().to_string()
    }

    fn split_into_batches_test(text: &str) -> Vec<String> {
        if text.len() <= MAX_BATCH_SIZE {
            return vec![text.to_string()];
        }

        let mut batches = Vec::new();
        let mut current_batch = String::new();
        let sentence_pattern = regex::Regex::new(r"([.!?]+\s+)").unwrap();
        let mut last_end = 0;

        for mat in sentence_pattern.find_iter(text) {
            let sentence = &text[last_end..mat.end()];
            if !current_batch.is_empty() && current_batch.len() + sentence.len() > MAX_BATCH_SIZE {
                batches.push(current_batch.trim().to_string());
                current_batch = String::new();
            }
            current_batch.push_str(sentence);
            last_end = mat.end();
        }

        // Handle remaining text after last sentence boundary
        if last_end < text.len() {
            let remaining = &text[last_end..];

            // If we have a current batch and adding remaining would exceed limit
            if !current_batch.is_empty() && current_batch.len() + remaining.len() > MAX_BATCH_SIZE {
                batches.push(current_batch.trim().to_string());
                current_batch = String::new();
            }

            // If remaining text itself is too large, split it by characters
            if remaining.len() > MAX_BATCH_SIZE {
                let chars: Vec<char> = remaining.chars().collect();
                for chunk in chars.chunks(MAX_BATCH_SIZE) {
                    batches.push(chunk.iter().collect());
                }
            } else {
                current_batch.push_str(remaining);
            }
        }

        // Add any remaining batch
        if !current_batch.is_empty() {
            batches.push(current_batch.trim().to_string());
        }

        batches
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
    fn test_split_into_batches_small_text() {
        let text = "This is a short text.";
        let batches = split_into_batches_test(text);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], text);
    }

    #[test]
    fn test_split_into_batches_respects_max_size() {
        // Create text larger than MAX_BATCH_SIZE
        let sentence = "This is a sentence. ";
        let text = sentence.repeat(200); // Will be > 3000 chars
        let batches = split_into_batches_test(&text);

        assert!(batches.len() > 1, "Text should be split into multiple batches");

        // All batches should be <= MAX_BATCH_SIZE
        for batch in &batches {
            assert!(
                batch.len() <= MAX_BATCH_SIZE,
                "Batch size {} exceeds MAX_BATCH_SIZE {}",
                batch.len(),
                MAX_BATCH_SIZE
            );
        }
    }

    #[test]
    fn test_split_into_batches_respects_sentence_boundaries() {
        let text = "First sentence. Second sentence. Third sentence.";
        let batches = split_into_batches_test(text);

        // Text is small, should be single batch
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], text);
    }

    #[test]
    fn test_split_into_batches_multiple_punctuation() {
        let text = "Question? Answer! Statement. Exclamation!";
        let batches = split_into_batches_test(text);
        assert_eq!(batches.len(), 1); // Small enough for one batch
    }

    #[test]
    fn test_split_into_batches_no_punctuation() {
        // Text without sentence boundaries should be split by characters
        let text = "a".repeat(MAX_BATCH_SIZE + 500);
        let batches = split_into_batches_test(&text);

        assert!(batches.len() >= 2, "Should split text without punctuation, got {} batches", batches.len());
        for (i, batch) in batches.iter().enumerate() {
            assert!(batch.len() <= MAX_BATCH_SIZE, "Batch {} has length {}", i, batch.len());
        }
    }

    #[test]
    fn test_split_into_batches_preserves_content() {
        let sentence = "This is sentence number X. ";
        let text = sentence.repeat(200);
        let batches = split_into_batches_test(&text);

        // Reconstruct and verify all content is preserved
        // Need to handle trimming that might remove spaces between batches
        let reconstructed = batches.join(" ");
        let original_words: Vec<&str> = text.split_whitespace().collect();
        let reconstructed_words: Vec<&str> = reconstructed.split_whitespace().collect();

        assert_eq!(
            original_words.len(),
            reconstructed_words.len(),
            "Word count should be preserved. Original: {}, Reconstructed: {}",
            original_words.len(),
            reconstructed_words.len()
        );
    }

    #[test]
    fn test_split_into_batches_edge_case_exactly_max_size() {
        let text = "a".repeat(MAX_BATCH_SIZE);
        let batches = split_into_batches_test(&text);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), MAX_BATCH_SIZE);
    }

    #[test]
    fn test_split_into_batches_edge_case_one_over_max_size() {
        let text = "a".repeat(MAX_BATCH_SIZE + 1);
        let batches = split_into_batches_test(&text);
        assert!(batches.len() >= 2, "Expected at least 2 batches, got {}", batches.len());
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
        let text = "Esto es una prueba en español. El rápido zorro marrón salta sobre el perro perezoso.";
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
        let text = "Questo è un test in italiano. La volpe marrone veloce salta sopra il cane pigro.";
        let language = detector.detect_language_of(text);
        assert_eq!(language, Some(Language::Italian));
    }

    #[test]
    fn test_detect_language_portuguese() {
        let detector = LanguageDetectorBuilder::from_all_languages().build();
        let text = "Este é um teste em português. A rápida raposa marrom salta sobre o cão preguiçoso.";
        let language = detector.detect_language_of(text);
        assert_eq!(language, Some(Language::Portuguese));
    }
}
