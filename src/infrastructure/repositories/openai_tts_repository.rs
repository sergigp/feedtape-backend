use super::tts_repository::TtsRepository;
use crate::domain::tts::LanguageCode;
use async_openai::{
    config::OpenAIConfig,
    types::{CreateSpeechRequest, SpeechModel, Voice},
    Client,
};
use async_trait::async_trait;
use std::sync::Arc;

/// OpenAI has a limit of 4096 characters per request
const MAX_BATCH_SIZE: usize = 4096;

/// OpenAI TTS implementation of TTS repository
pub struct OpenAiTtsRepository {
    client: Arc<Client<OpenAIConfig>>,
    model: String,
    default_voice: String,
}

impl OpenAiTtsRepository {
    pub fn new(client: Arc<Client<OpenAIConfig>>, model: String, default_voice: String) -> Self {
        Self {
            client,
            model,
            default_voice,
        }
    }

    /// Select the appropriate OpenAI voice for a language
    /// Based on voice characteristics that suit each language
    fn get_voice_for_language(&self, language: LanguageCode) -> String {
        match language {
            LanguageCode::English => "alloy".to_string(), // Neutral American accent
            LanguageCode::Spanish => "echo".to_string(),  // Warm, clear for Spanish
            LanguageCode::French => "nova".to_string(),   // Soft, suitable for French
            LanguageCode::German => "onyx".to_string(),   // Clear, authoritative
            LanguageCode::Italian => "fable".to_string(), // Expressive for Italian
            LanguageCode::Portuguese => "shimmer".to_string(), // Clear articulation
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

    /// Call OpenAI TTS API to synthesize a single text batch
    async fn call_openai(&self, text: &str, voice: &str) -> Result<Vec<u8>, String> {
        tracing::info!(
            model = %self.model,
            voice = voice,
            text_length = text.len(),
            text_preview = &text[..text.len().min(200)],
            "Calling OpenAI TTS API"
        );

        // Parse model string to SpeechModel enum
        let model = match self.model.as_str() {
            "tts-1" => SpeechModel::Tts1,
            "tts-1-hd" => SpeechModel::Tts1Hd,
            other => SpeechModel::Other(other.to_string()),
        };

        // Parse voice string to Voice enum
        let voice_enum = match voice.to_lowercase().as_str() {
            "alloy" => Voice::Alloy,
            "echo" => Voice::Echo,
            "fable" => Voice::Fable,
            "onyx" => Voice::Onyx,
            "nova" => Voice::Nova,
            "shimmer" => Voice::Shimmer,
            _ => Voice::Alloy, // Default fallback
        };

        let request = CreateSpeechRequest {
            model,
            input: text.to_string(),
            voice: voice_enum,
            response_format: None, // Defaults to MP3
            speed: None,           // Defaults to 1.0
        };

        let response = self
            .client
            .audio()
            .speech(request)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    model = %self.model,
                    voice = voice,
                    text_length = text.len(),
                    "OpenAI TTS API call failed"
                );
                format!("OpenAI TTS error: {}", e)
            })?;

        let audio_bytes = response.bytes.to_vec();
        tracing::debug!(
            audio_size = audio_bytes.len(),
            "OpenAI TTS audio received successfully"
        );

        Ok(audio_bytes)
    }

    /// Synthesize multiple text batches and merge the audio results in order
    async fn synthesize_batches(
        &self,
        batches: &[String],
        voice: &str,
    ) -> Result<Vec<u8>, String> {
        let mut merged_audio = Vec::new();

        for (index, batch) in batches.iter().enumerate() {
            tracing::info!(
                batch_index = index,
                batch_size = batch.len(),
                "Synthesizing batch"
            );

            let audio_data = self.call_openai(batch, voice).await?;
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
impl TtsRepository for OpenAiTtsRepository {
    async fn synthesize(&self, text: &str, language: LanguageCode) -> Result<Vec<u8>, String> {
        let start_time = std::time::Instant::now();

        // Select voice based on language (or use default if configured)
        let voice = if self.default_voice.is_empty() {
            self.get_voice_for_language(language)
        } else {
            self.default_voice.clone()
        };

        tracing::info!(
            language = %language,
            voice = %voice,
            model = %self.model,
            text_length = text.len(),
            "Starting OpenAI TTS synthesis"
        );

        // Split text into batches based on OpenAI's limitations
        let batches = self.split_into_batches(text);
        tracing::info!(
            batch_count = batches.len(),
            text_length = text.len(),
            "Text split into batches"
        );

        // Synthesize each batch and merge results
        let audio_data = self.synthesize_batches(&batches, &voice).await?;

        let duration = start_time.elapsed();
        let characters_count = text.len();
        let throughput_chars_per_sec = if duration.as_secs_f64() > 0.0 {
            characters_count as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        tracing::info!(
            provider = "openai",
            model = %self.model,
            voice = %voice,
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
        let text = sentence.repeat(300); // Will be > 4096 chars
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
