# End-to-End Tests

This directory contains comprehensive end-to-end tests for the FeedTape Backend API.

## Prerequisites

- Docker must be installed and running (for testcontainers)
- Rust toolchain installed
- Database migrations in `migrations/` directory

## Running Tests

Run all tests:
```bash
cargo test --test '*'
```

Run specific test file:
```bash
cargo test --test test_feeds
cargo test --test test_auth
cargo test --test test_user
cargo test --test test_tts
cargo test --test test_health
```

Run with verbose output:
```bash
cargo test --test '*' -- --nocapture
```

Run a specific test:
```bash
cargo test --test test_feeds it_should_create_a_new_feed
```

## Test Organization

- `helpers/` - Test utilities and fixtures
  - `mod.rs` - Test context and setup
  - `api_client.rs` - HTTP client for testing
  - `fixtures.rs` - Database fixtures and test data
  - `assertions.rs` - Common assertion helpers
  - `aws_mocks.rs` - AWS service mocks

- Test files:
  - `test_auth.rs` - Authentication and token management
  - `test_feeds.rs` - Feed CRUD operations
  - `test_user.rs` - User profile and settings
  - `test_tts.rs` - Text-to-speech functionality
  - `test_health.rs` - Health check endpoints

## Test Features

### Authentication Tests
- JWT validation
- Token refresh
- Logout (single and all sessions)
- Expired token handling
- Malformed token rejection

### Feed Tests
- Create, read, update, delete feeds
- Feed limits for free/pro tiers
- Duplicate URL prevention
- User isolation (feeds are private)
- URL validation

### User Tests
- Get user profile
- Update settings (voice, speed, language, quality)
- Subscription status
- Usage statistics
- Setting validation

### TTS Tests
- Text synthesis
- Language auto-detection
- Voice and quality settings
- Usage limits enforcement
- Pro tier features

### Health Tests
- Basic health check
- Readiness with service status
- No authentication required
- Performance validation

## Test Patterns

All tests follow these patterns:

1. **Naming**: Tests start with `it_should_` followed by the behavior being tested
2. **Isolation**: Each test creates its own test context with a fresh database
3. **Serial Execution**: Tests run sequentially to avoid conflicts
4. **Behavior Focus**: Tests validate API behavior, not implementation details

## Mocking

- **Database**: Uses testcontainers to spin up real PostgreSQL instances
- **AWS Services**: Mocked using a fake endpoint configuration
- **Time**: Tests use actual system time (consider using time mocking for deterministic tests)

## Common Issues

1. **Docker not running**: Ensure Docker Desktop is running before tests
2. **Port conflicts**: Tests use random ports, but ensure no conflicts with development servers
3. **Slow tests**: First run downloads Docker images; subsequent runs are faster
4. **Database migrations**: Ensure migrations are up-to-date in `migrations/`

## Adding New Tests

1. Create a new test file: `test_feature.rs`
2. Add `mod test_feature;` to `mod.rs`
3. Use the test helpers:
   ```rust
   mod helpers;
   use helpers::{TestContext, generate_test_jwt};

   #[tokio::test]
   #[serial]
   async fn it_should_do_something() {
       let ctx = TestContext::new().await.unwrap();
       // Test implementation
   }
   ```

## CI/CD Integration

For CI/CD pipelines, ensure:
- Docker is available in the CI environment
- Set `DOCKER_HOST` if using remote Docker
- Consider using `--release` mode for faster execution
- Cache Docker images between runs