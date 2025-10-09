use lingua::{Language, LanguageDetectorBuilder};

/// Detect the language of the given text
/// Returns ISO 639-1 language code (en, es, fr, de, it, pt) or defaults to "es"
pub fn detect_language(text: &str) -> String {
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
        match language {
            Language::English => "en".to_string(),
            Language::Spanish => "es".to_string(),
            Language::French => "fr".to_string(),
            Language::German => "de".to_string(),
            Language::Italian => "it".to_string(),
            Language::Portuguese => "pt".to_string(),
        }
    } else {
        "es".to_string() // Default to Spanish if detection fails
    }
}

/// Get the appropriate Polly voice ID for a language and quality
pub fn get_voice_for_language(language: &str, quality: &str) -> &'static str {
    // Return voice based on language and quality (neural vs standard)
    match (language, quality) {
        // Neural voices
        ("en", "neural") => "Joanna",
        ("es", "neural") => "Lupe",   // Lucia doesn't support neural
        ("fr", "neural") => "Lea",
        ("de", "neural") => "Vicki",
        ("it", "neural") => "Bianca",
        ("pt", "neural") => "Ines",

        // Standard voices (fallback)
        ("en", _) => "Joanna",
        ("es", _) => "Lucia",
        ("fr", _) => "Lea",
        ("de", _) => "Vicki",
        ("it", _) => "Bianca",
        ("pt", _) => "Ines",

        // Default
        _ => "Lucia",
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
        "Lupe", "Pedro", "Sergio",
        // French
        "Lea", "Remi",
        // German
        "Vicki", "Daniel",
        // Italian
        "Bianca", "Adriano",
        // Portuguese
        "Ines", "Camila", "Vitoria", "Thiago",
        // Japanese
        "Takumi", "Kazuha", "Tomoko",
        // Korean
        "Seoyeon",
        // Mandarin Chinese
        "Zhiyu",
        // Arabic
        "Hala", "Zayd",
    ];

    NEURAL_VOICES.contains(&voice)
}
