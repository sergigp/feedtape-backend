use super::tts_repository::TtsRepository;
use crate::domain::tts::LanguageCode;
use async_trait::async_trait;
use aws_sdk_polly::{
    types::{Engine, OutputFormat, VoiceId},
    Client as PollyClient,
};
use std::sync::Arc;

/// AWS Polly has a limit of 3000 characters per request
const MAX_BATCH_SIZE: usize = 3000;

/// AWS Polly implementation of TTS repository
pub struct PollyTtsRepository {
    polly_client: Arc<PollyClient>,
}

impl PollyTtsRepository {
    pub fn new(polly_client: Arc<PollyClient>) -> Self {
        Self { polly_client }
    }

    /// Select the appropriate Polly voice for a language
    fn get_voice_for_language(language: LanguageCode) -> &'static str {
        match language {
            LanguageCode::English => "Joanna",
            LanguageCode::Spanish => "Lupe",
            LanguageCode::French => "Lea",
            LanguageCode::German => "Vicki",
            LanguageCode::Italian => "Bianca",
            LanguageCode::Portuguese => "Ines",
        }
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

    /// Call AWS Polly to synthesize a single text batch
    async fn call_polly(
        &self,
        text: &str,
        language_code: LanguageCode,
    ) -> Result<Vec<u8>, String> {
        // Select voice based on detected language (always use neural)
        let voice_name = Self::get_voice_for_language(language_code);
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
        let result = self
            .polly_client
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
                format!("AWS Polly error: {:?}", e)
            })?;

        tracing::debug!("AWS Polly synthesize_speech successful, reading audio stream");

        // Get audio stream
        let audio_stream = result.audio_stream.collect().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to collect audio stream from Polly response");
            format!("Failed to read audio stream: {}", e)
        })?;

        let audio_bytes = audio_stream.into_bytes().to_vec();
        tracing::debug!(
            audio_size = audio_bytes.len(),
            "Audio stream collected successfully"
        );

        Ok(audio_bytes)
    }

    /// Synthesize multiple text batches and merge the audio results in order
    async fn synthesize_batches(
        &self,
        batches: &[String],
        language_code: LanguageCode,
    ) -> Result<Vec<u8>, String> {
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
}

#[async_trait]
impl TtsRepository for PollyTtsRepository {
    async fn synthesize(&self, text: &str, language: LanguageCode) -> Result<Vec<u8>, String> {
        let start_time = std::time::Instant::now();

        // Split text into batches based on Polly's limitations
        let batches = self.split_into_batches(text);
        tracing::info!(
            batch_count = batches.len(),
            text_length = text.len(),
            "Text split into batches"
        );

        // Synthesize each batch and merge results
        let audio_data = self.synthesize_batches(&batches, language).await?;

        let duration = start_time.elapsed();
        let characters_count = text.len();
        let throughput_chars_per_sec = if duration.as_secs_f64() > 0.0 {
            characters_count as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        tracing::info!(
            provider = "polly",
            latency_ms = duration.as_millis(),
            latency_secs = duration.as_secs_f64(),
            characters_count = characters_count,
            batch_count = batches.len(),
            audio_size_bytes = audio_data.len(),
            throughput_chars_per_sec = format!("{:.2}", throughput_chars_per_sec),
            "TTS synthesis completed"
        );

        Ok(audio_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(
            batches.len() > 1,
            "Text should be split into multiple batches"
        );

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

        assert!(
            batches.len() >= 2,
            "Should split text without punctuation, got {} batches",
            batches.len()
        );
        for (i, batch) in batches.iter().enumerate() {
            assert!(
                batch.len() <= MAX_BATCH_SIZE,
                "Batch {} has length {}",
                i,
                batch.len()
            );
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
        assert!(
            batches.len() >= 2,
            "Expected at least 2 batches, got {}",
            batches.len()
        );
    }
}
