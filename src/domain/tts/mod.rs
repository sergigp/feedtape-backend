pub mod error;
pub mod language;
pub mod service;

pub use error::TtsServiceError;
pub use language::{detect_language, get_voice_for_language};
use serde::{Deserialize, Serialize};
pub use service::{TtsService, TtsServiceApi, TtsSynthesisResult};

/// Request for POST /api/tts/synthesize
#[derive(Debug, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}
