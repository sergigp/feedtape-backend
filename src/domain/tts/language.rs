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

/// Get the appropriate Polly voice ID for a language and gender
pub fn get_voice_for_language(language: &str, _quality: &str) -> &'static str {
    // For MVP, using one voice per language (can expand later)
    match language {
        "en" => "Joanna", // English (US, Female, Neural)
        "es" => "Lucia",  // Spanish (Female, Neural)
        "fr" => "Lea",    // French (Female, Neural)
        "de" => "Vicki",  // German (Female, Neural)
        "it" => "Bianca", // Italian (Female, Neural)
        "pt" => "Ines",   // Portuguese (Female, Neural)
        _ => "Lucia",     // Default to Spanish
    }
}
