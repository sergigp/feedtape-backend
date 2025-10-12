// End-to-end integration tests for FeedTape Backend API
//
// These tests use a shared testcontainers PostgreSQL instance with a database
// pool for test isolation. Each test receives its own isolated database from
// the pool, allowing tests to run in parallel without conflicts.
//
// Architecture:
// - One shared PostgreSQL container for the entire test suite
// - Database pool creates/manages isolated databases (test_db_<uuid>)
// - Each test gets a unique database via test-context lifecycle hooks
// - Databases are cleaned and recycled after test completion
//
// Tests run in parallel by default, significantly improving test performance.

mod helpers;
mod test_auth;
mod test_feed_suggestions;
mod test_feeds;
mod test_health;
mod test_oauth;
mod test_tts;
mod test_user;
