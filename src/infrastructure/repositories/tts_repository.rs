use crate::domain::tts::LanguageCode;
use async_trait::async_trait;

/// Repository for TTS synthesis operations.
/// Abstracts the underlying TTS provider (AWS Polly, OpenAI, ElevenLabs, etc.)
///
/// Implementations are responsible for:
/// - Handling provider-specific text length limitations
/// - Splitting text into batches if needed
/// - Merging audio chunks into a single audio stream
/// - Provider-specific voice selection
#[async_trait]
pub trait TtsRepository: Send + Sync {
    /// Synthesize text to speech for a given language
    ///
    /// Returns merged audio data ready for playback (MP3 format)
    ///
    /// # Arguments
    /// * `text` - The cleaned text to synthesize (no HTML, normalized whitespace)
    /// * `language` - The target language for synthesis
    ///
    /// # Errors
    /// Returns error if synthesis fails or provider is unavailable
    async fn synthesize(&self, text: &str, language: LanguageCode) -> Result<Vec<u8>, String>;
}
