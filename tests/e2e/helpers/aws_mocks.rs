use aws_sdk_polly::Client as PollyClient;

pub async fn create_mock_polly_client() -> PollyClient {
    // Create a mock Polly client configuration
    let config = aws_sdk_polly::Config::builder()
        .behavior_version(aws_sdk_polly::config::BehaviorVersion::latest())
        .region(aws_sdk_polly::config::Region::new("us-east-1"))
        .endpoint_url("http://localhost:9999") // Non-existent endpoint for testing
        .build();

    PollyClient::from_conf(config)
}

#[allow(dead_code)]
pub fn mock_audio_bytes() -> Vec<u8> {
    // Minimal valid MP3 file (silence)
    vec![
        0xFF, 0xFB, 0x90, 0x00, // MP3 frame header
        0x00, 0x00, 0x00, 0x00, // Some padding
    ]
}