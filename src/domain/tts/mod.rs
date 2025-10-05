pub mod dto;
pub mod language;
pub mod service;

pub use dto::TtsRequest;
pub use language::{detect_language, get_voice_for_language};
pub use service::TtsService;
