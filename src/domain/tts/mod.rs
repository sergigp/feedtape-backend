pub mod error;
pub mod language;
pub mod service;

pub use error::TtsServiceError;
pub use language::{detect_language, get_voice_for_language, LanguageCode};
pub use service::{TtsService, TtsServiceApi, TtsSynthesisResult};
