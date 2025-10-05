# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FeedTape Backend is a Rust-based REST API server for converting RSS feeds to audio using AWS Polly. Built with Axum framework and PostgreSQL, it follows a layered architecture pattern separating domain logic, infrastructure, and controllers.

## Development Commands

### Build & Run
```bash
# Run in development with auto-reload
cargo watch -x run

# Run without auto-reload
cargo run

# Build for release
cargo build --release

# Check compilation without building
cargo check
```

### Testing
```bash
# Run all tests (unit + integration + e2e)
cargo test

# Run specific test file
cargo test --test test_feeds
cargo test --test test_auth
cargo test --test test_user
cargo test --test test_tts

# Run with output
cargo test -- --nocapture

# Run a single test
cargo test --test test_feeds it_should_create_a_new_feed
```

### Database
```bash
# Start PostgreSQL (required for development)
docker-compose up -d

# Stop database
docker-compose down

# Reset database (wipe data)
docker-compose down
docker volume rm feedtape-backend_postgres_data
docker-compose up -d
```

### Linting & Formatting
```bash
# Run clippy linter
cargo clippy

# Format code
cargo fmt

# Format check (CI)
cargo fmt -- --check
```

## Architecture

The codebase follows a Domain-Driven Design with clear separation of concerns:

### Layer Structure
- **`src/controllers/`** - HTTP request handlers that orchestrate service calls
- **`src/domain/`** - Business logic organized by feature (auth, feed, tts, user)
  - Each domain module contains: `service.rs`, `dto.rs`, `model.rs`
- **`src/infrastructure/`** - External integrations and implementations
  - `repositories/` - Database access layer using SQLx
  - `auth/` - JWT middleware and request ID tracking
  - `config/` - Environment configuration
  - `db/` - Database pool management
  - `http/` - Server setup and routing

### Key Patterns
1. **Repository Pattern**: All database operations go through repositories that return domain models
2. **Service Layer**: Business logic lives in services that coordinate repositories and external clients
3. **DTO Pattern**: Separate DTOs for API requests/responses vs domain models
4. **Dependency Injection**: Dependencies are injected via Arc<> in main.rs startup

### Database Schema
- Uses SQLx with compile-time query verification
- Migrations in `migrations/` directory (currently single initial schema)
- Main tables: `users`, `feeds`, `refresh_tokens`, `usage_tracking`

### Authentication Flow
- JWT-based with access tokens (1 hour) and refresh tokens (30 days)
- Middleware extracts and validates tokens on protected routes
- User context available via Extension in handlers

## Testing Strategy

### E2E Tests (`tests/e2e/`)
- Uses testcontainers to spin up real PostgreSQL instances
- Each test gets isolated database via `TestContext`
- Tests marked with `#[serial]` to avoid conflicts
- Helpers in `tests/e2e/helpers/` provide fixtures and API client

### Running E2E Tests in CI
The project uses GitHub Actions with testcontainers. Key requirements:
- Docker must be available
- Tests run with `cargo nextest` for better parallelism control
- Pre-pulls common Docker images to reduce flakiness

## Environment Configuration

Required environment variables (see `.env.example`):
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Secret for signing JWTs (generate with `openssl rand -base64 32`)
- `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` - AWS credentials for Polly

The project uses `dotenvy` to load `.env` files in development.

## AWS Polly Integration

- TTS service in `src/domain/tts/service.rs` handles synthesis
- Language auto-detection via lingua-rs (supports 6 languages)
- Voice mapping per language defined in service
- Usage tracking enforced before synthesis
- Pro tier gets neural voices, free tier gets standard

## Common Development Tasks

### Adding a New API Endpoint
1. Define DTOs in `src/domain/{feature}/dto.rs`
2. Implement business logic in `src/domain/{feature}/service.rs`
3. Add repository methods if needed in `src/infrastructure/repositories/`
4. Create handler in `src/controllers/{feature}.rs`
5. Register route in `src/infrastructure/http/mod.rs`
6. Write E2E test in `tests/e2e/test_{feature}.rs`

### Modifying Database Schema
1. Create new migration file in `migrations/`
2. Update repository structs and queries
3. Run tests to verify SQLx compile-time checks pass

### Debugging Test Failures
- Check Docker is running for testcontainers
- Use `--nocapture` to see print statements
- Look for database constraint violations in logs
- Verify AWS mock endpoint is configured in tests