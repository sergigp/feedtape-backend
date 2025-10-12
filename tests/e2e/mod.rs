// End-to-end integration tests for FeedTape Backend API
//
// These tests use testcontainers to spin up a PostgreSQL database
// and test the complete API flow including authentication, feeds,
// users, and TTS functionality.
//
// Tests are marked with #[serial] to run sequentially to avoid
// database conflicts during test execution.

mod helpers;
mod test_auth;
mod test_feed_suggestions;
mod test_feeds;
mod test_health;
mod test_oauth;
mod test_tts;
mod test_user;
