# FeedTape Backend Constitution

<!--
Sync Impact Report:
- Version change: none → 1.0.0
- New sections added: All initial sections
- Modified principles: None (initial version)
- Templates requiring updates:
  ✅ .specify/templates/plan-template.md - Constitution Check section aligned
  ✅ .specify/templates/spec-template.md - Requirement structures aligned
  ✅ .specify/templates/tasks-template.md - Test and task organization aligned
- Follow-up TODOs: None
-->

## Core Principles

### I. Domain-Driven Architecture (NON-NEGOTIABLE)

The codebase MUST follow a strict three-layer architecture separating concerns:

- **Controllers** (`src/controllers/`) handle HTTP requests and orchestrate service calls only
- **Domain** (`src/domain/`) contains business logic organized by feature (auth, feed, tts, user)
  - Each domain module's `mod.rs` contains entities and domain objects
  - `service.rs` contains business logic and orchestrates repositories
- **Infrastructure** (`src/infrastructure/`) provides external integrations (repositories, AWS, auth middleware)

**Rationale**: This separation ensures business logic remains testable and independent of HTTP and database concerns. It prevents tight coupling and makes the system easier to evolve.

**Rules**:
- Controllers MUST NOT contain business logic
- Domain services MUST NOT directly access HTTP or database layers
- Repositories MUST return domain models, not database-specific structs
- No circular dependencies between layers

### II. Repository Pattern (NON-NEGOTIABLE)

All database operations MUST go through repositories that return domain models.

**Rationale**: Repositories abstract database access, enabling independent testing of business logic and potential database migration without rewriting domain code.

**Rules**:
- All database queries live in `src/infrastructure/repositories/`
- Repositories return domain entities defined in `src/domain/{feature}/mod.rs`
- Services receive repositories as dependencies
- No direct SQLx queries outside repositories

### III. Dependency Injection & Minimal Config

Services MUST receive only the specific configuration values they need, not the entire `Config` struct.

**Rationale**: Reduces coupling and makes dependencies explicit. Services should declare exactly what they need.

**Rules**:
- Services receive individual config values (e.g., `aws_region: String`, `jwt_secret: String`)
- Avoid passing entire `Config` struct to services
- Dependencies must be explicit in function signatures

### IV. Test-First with Real Infrastructure (E2E Focus)

E2E tests MUST use real infrastructure (testcontainers for PostgreSQL) and MUST be isolated.

**Rationale**: Compile-time query verification with SQLx requires real database. Integration tests catch more bugs than mocks. Isolation prevents flakiness.

**Rules**:
- E2E tests in `tests/e2e/` use testcontainers
- Each test gets isolated database via `TestContext`
- Tests marked `#[serial]` when needed to avoid conflicts
- Helper modules in `tests/e2e/helpers/` provide fixtures
- Tests must clean up after themselves

### V. Compile-Time Safety with SQLx

Database queries MUST be verified at compile time using SQLx.

**Rationale**: Catches SQL errors before runtime. Ensures schema and queries stay in sync.

**Rules**:
- All queries use SQLx macros (`query!`, `query_as!`)
- Migrations in `migrations/` directory
- Run `cargo check` to verify queries before commits
- No raw SQL strings without compile-time verification

### VI. Security & Authentication Standards

JWT-based authentication MUST follow secure patterns with proper token management.

**Rationale**: Protects user data and prevents unauthorized access.

**Rules**:
- Access tokens expire in 1 hour
- Refresh tokens expire in 30 days and are stored in database
- Middleware extracts and validates tokens on protected routes
- User context available via `Extension` in handlers
- JWT secrets MUST be strong (minimum 32 bytes)

### VII. Observability & Request Tracking

All requests MUST be traceable with structured logging.

**Rationale**: Enables debugging in production and understanding user behavior.

**Rules**:
- Request ID middleware tracks all requests
- Use `tracing` crate with structured logs
- Log format: `pretty` in dev, `json` in production
- Log level configurable via `RUST_LOG` environment variable

## Architecture Constraints

### Technology Stack (LOCKED)

**Required Stack**:
- **Language**: Rust 1.70+
- **Framework**: Axum 0.7
- **Database**: PostgreSQL 15
- **ORM**: SQLx (compile-time verification)
- **TTS**: AWS Polly
- **Language Detection**: Lingua-rs
- **Authentication**: JWT (jsonwebtoken crate)
- **Logging**: Tracing

**Rationale**: Stack chosen for type safety, performance, and compile-time guarantees. Changes require constitution amendment.

### External Dependencies

**AWS Polly Integration**:
- TTS service in `src/domain/tts/service.rs` handles synthesis
- Language auto-detection via lingua-rs (supports 6 languages: EN, ES, FR, DE, IT, PT)
- Voice mapping per language defined in service
- Usage tracking enforced before synthesis
- Pro tier gets neural voices, free tier gets standard

**Database Requirements**:
- PostgreSQL 15+ required
- Docker Compose for local development
- Uses SQLx migrations (single schema currently)
- Main tables: `users`, `feeds`, `refresh_tokens`, `usage_tracking`

## Development Workflow

### API Endpoint Development Process

When adding a new API endpoint, follow this sequence:

1. Define entities and domain objects in `src/domain/{feature}/mod.rs`
2. Implement business logic in `src/domain/{feature}/service.rs`
3. Add repository methods in `src/infrastructure/repositories/` if needed
4. Create handler in `src/controllers/{feature}.rs`
5. Register route in `src/infrastructure/http/mod.rs`
6. Write E2E test in `tests/e2e/test_{feature}.rs`

### Database Schema Changes

1. Create new migration file in `migrations/`
2. Update repository structs and queries
3. Run tests to verify SQLx compile-time checks pass
4. Document schema changes in commit message

### Code Quality Gates

**Before Commits**:
- `cargo check` - Compile-time verification passes
- `cargo clippy` - No linter warnings
- `cargo fmt -- --check` - Code formatted
- `cargo test` - All tests pass

**CI/CD Requirements**:
- GitHub Actions runs all checks
- Testcontainers for E2E tests
- Docker images pre-pulled to reduce flakiness
- Tests run with `cargo nextest` for better parallelism

## Governance

### Constitution Authority

This constitution supersedes all other development practices. When conflicts arise between this document and other guidance, the constitution prevails.

### Amendment Process

1. Propose amendment with rationale and impact analysis
2. Update version according to semantic versioning:
   - **MAJOR**: Backward incompatible governance/principle removal or redefinition
   - **MINOR**: New principle/section added or materially expanded guidance
   - **PATCH**: Clarifications, wording, typo fixes, non-semantic refinements
3. Update dependent templates (plan, spec, tasks) to align with changes
4. Document amendment in Sync Impact Report
5. Update `LAST_AMENDED_DATE` to current date

### Compliance Review

- All PRs MUST verify compliance with architecture principles
- Code reviews MUST check for violations of layer separation
- Complexity additions MUST be justified against principles
- Use `CLAUDE.md` for runtime development guidance (tool-specific)

### Versioning

**Current Version**: 1.0.0
**Ratified**: 2025-10-11
**Last Amended**: 2025-10-11
