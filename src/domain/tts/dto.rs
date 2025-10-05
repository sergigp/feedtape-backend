use serde::{Deserialize, Serialize};

/// Request for POST /api/tts/synthesize
#[derive(Debug, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
}
