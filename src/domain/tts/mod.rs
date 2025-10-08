pub mod language;
pub mod service;

pub use language::{detect_language, get_voice_for_language};
use serde::{Deserialize, Serialize};
pub use service::TtsService;

/// Request for POST /api/tts/synthesize
#[derive(Debug, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
}
