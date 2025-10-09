use lingua::{Language, LanguageDetectorBuilder};
use serde::{Deserialize, Serialize};

/// ISO 639-1 language codes supported by the TTS system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageCode {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "pt")]
    Portuguese,
}

impl LanguageCode {
    /// Get the ISO 639-1 code as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageCode::English => "en",
            LanguageCode::Spanish => "es",
            LanguageCode::French => "fr",
            LanguageCode::German => "de",
            LanguageCode::Italian => "it",
            LanguageCode::Portuguese => "pt",
        }
    }

    /// Convert lingua Language to LanguageCode
    pub fn from_lingua(language: Language) -> Self {
        match language {
            Language::English => LanguageCode::English,
            Language::Spanish => LanguageCode::Spanish,
            Language::French => LanguageCode::French,
            Language::German => LanguageCode::German,
            Language::Italian => LanguageCode::Italian,
            Language::Portuguese => LanguageCode::Portuguese,
        }
    }
}

impl std::fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detect the language of the given text
/// Returns LanguageCode or defaults to Spanish
pub fn detect_language(text: &str) -> LanguageCode {
    // Build detector with our supported languages
    let languages = vec![
        Language::English,
        Language::Spanish,
        Language::French,
        Language::German,
        Language::Italian,
        Language::Portuguese,
    ];

    let detector = LanguageDetectorBuilder::from_languages(&languages).build();

    // Detect language
    if let Some(language) = detector.detect_language_of(text) {
        LanguageCode::from_lingua(language)
    } else {
        LanguageCode::Spanish // Default to Spanish if detection fails
    }
}

/// Get the appropriate Polly voice ID for a language and quality
pub fn get_voice_for_language(language: LanguageCode) -> &'static str {
    match language {
        LanguageCode::English => "Joanna",
        LanguageCode::Spanish => "Lupe",
        LanguageCode::French => "Lea",
        LanguageCode::German => "Vicki",
        LanguageCode::Italian => "Bianca",
        LanguageCode::Portuguese => "Ines",
    }
}

/// Check if a voice supports neural engine
pub fn is_voice_neural_compatible(voice: &str) -> bool {
    // List of voices that support neural engine
    // Based on AWS Polly documentation
    const NEURAL_VOICES: &[&str] = &[
        // English
        "Joanna", "Matthew", "Ivy", "Kendra", "Kimberly", "Salli", "Joey", "Justin", "Kevin",
        // Spanish
        "Lupe", "Pedro", "Sergio", // French
        "Lea", "Remi", // German
        "Vicki", "Daniel", // Italian
        "Bianca", "Adriano", // Portuguese
        "Ines", "Camila", "Vitoria", "Thiago", // Japanese
        "Takumi", "Kazuha", "Tomoko",  // Korean
        "Seoyeon", // Mandarin Chinese
        "Zhiyu",   // Arabic
        "Hala", "Zayd",
    ];

    NEURAL_VOICES.contains(&voice)
}
