# Implementation Plan: Feed Suggestions by Category

**Branch**: `001-i-want-to` | **Date**: 2025-10-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-i-want-to/spec.md`

**User Input**: "I would like to create a repository that contins a hardcoded list of categories and feeds with an interface, etc. With this we will be able to replace the implementation in the future if needed. Always return all categories and feeds correctly estructured"

## Summary

Create a feed suggestions system that helps new users discover content by providing curated RSS feed recommendations organized into 20 categories. The system will expose two REST endpoints: one to retrieve all available categories, and another to fetch feed suggestions filtered by category identifiers. The implementation will use a repository pattern with hardcoded data to enable future migration to a database or external service without changing the domain layer interface.

## Technical Context

**Language/Version**: Rust 1.70+
**Primary Dependencies**: Axum 0.7, SQLx (for consistency, though this feature won't use database)
**Storage**: In-memory hardcoded data structures (no database required for this feature)
**Testing**: cargo test with testcontainers for E2E tests
**Target Platform**: Linux server (deployed on Railway.app)
**Project Type**: Single backend API (existing Rust project)
**Performance Goals**: <500ms response time for all endpoints, support 1000+ concurrent users
**Constraints**:
- Endpoints must respond in under 500 milliseconds
- Must support 20 categories with 4 feeds each (80 total feed suggestions)
- Must deduplicate feeds when multiple categories are requested
- Must follow existing authentication patterns (JWT required)
**Scale/Scope**:
- 2 new REST endpoints
- 20 categories, 80 feed suggestions total
- Read-only operations (no mutations)
- Expected usage: ~100-1000 requests/day initially

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### ✅ I. Domain-Driven Architecture (NON-NEGOTIABLE)

**Status**: COMPLIANT

**Plan**:
- **Domain** (`src/domain/feed_suggestions/`):
  - `mod.rs`: Define `Category`, `FeedSuggestion` entities
  - `service.rs`: Business logic for retrieving categories and filtering suggestions
- **Infrastructure** (`src/infrastructure/repositories/`):
  - `feed_suggestions_repository.rs`: Repository with hardcoded data implementing trait interface
- **Controllers** (`src/controllers/`):
  - `feed_suggestions.rs`: HTTP handlers that call domain service

**Compliance**: Three-layer separation maintained. No business logic in controllers, no HTTP in domain.

---

### ✅ II. Repository Pattern (NON-NEGOTIABLE)

**Status**: COMPLIANT

**Plan**:
- Create `FeedSuggestionsRepository` trait in domain layer
- Implement trait with hardcoded data in infrastructure layer
- Service receives repository as dependency
- Future migration to database or external API only requires new trait implementation

**Compliance**: Repository abstracts data source. Domain depends on trait, not concrete implementation.

---

### ✅ III. Dependency Injection & Minimal Config

**Status**: COMPLIANT

**Plan**:
- Service receives repository reference, no config needed
- Controllers receive service as `Arc<FeedSuggestionsService>`
- No global state or config struct passed around

**Compliance**: No configuration needed for this feature. Repository is sole dependency.

---

### ✅ IV. Test-First with Real Infrastructure (E2E Focus)

**Status**: COMPLIANT

**Plan**:
- E2E tests in `tests/e2e/test_feed_suggestions.rs`
- Use testcontainers for consistency (even though feature doesn't use DB)
- Test authentication middleware integration
- Test multi-category filtering and deduplication
- Isolated test contexts per test

**Compliance**: E2E tests cover full HTTP request/response cycle with authentication.

---

### ✅ V. Compile-Time Safety with SQLx

**Status**: N/A (No database queries in this feature)

**Plan**:
- Feature uses in-memory data structures
- No SQLx queries needed
- No migrations required

**Compliance**: Not applicable. Feature intentionally avoids database to enable fast iteration.

---

### ✅ VI. Security & Authentication Standards

**Status**: COMPLIANT

**Plan**:
- Both endpoints require JWT authentication
- Use existing auth middleware
- User context extracted via `Extension<User>`
- No user-specific data (all suggestions are global)

**Compliance**: Follows existing JWT patterns. Endpoints are protected routes.

---

### ✅ VII. Observability & Request Tracking

**Status**: COMPLIANT

**Plan**:
- Use existing request ID middleware
- Structured logging with `tracing` crate
- Log category filters and response sizes
- Log any validation errors (invalid category IDs)

**Compliance**: Follows existing logging patterns with structured events.

---

### Summary

**Overall Status**: ✅ ALL CHECKS PASSED

No violations. Feature follows all constitutional principles. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```
specs/001-i-want-to/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: Research findings
├── data-model.md        # Phase 1: Entity definitions
├── quickstart.md        # Phase 1: Developer guide
├── contracts/           # Phase 1: API contracts
│   └── openapi.yaml
└── checklists/
    └── requirements.md  # Specification quality checklist
```

### Source Code (repository root)

```
src/
├── controllers/
│   └── feed_suggestions.rs         # NEW: HTTP handlers for feed suggestions
├── domain/
│   ├── feed_suggestions/           # NEW: Feed suggestions domain module
│   │   ├── mod.rs                  # NEW: Entities (Category, FeedSuggestion, repository trait)
│   │   └── service.rs              # NEW: Business logic
│   ├── feed/                       # EXISTING: User's personal feeds
│   ├── user/                       # EXISTING
│   ├── auth/                       # EXISTING
│   └── tts/                        # EXISTING
├── infrastructure/
│   ├── repositories/
│   │   └── feed_suggestions_repository.rs  # NEW: Hardcoded data implementation
│   ├── auth/                       # EXISTING: JWT middleware (reused)
│   └── http/
│       └── mod.rs                  # MODIFIED: Register new routes
└── main.rs                         # MODIFIED: Wire up new service

tests/
└── e2e/
    └── test_feed_suggestions.rs    # NEW: E2E tests for feed suggestions
```

**Structure Decision**: Single Rust project structure following existing conventions. New domain module `feed_suggestions` added alongside existing modules (`feed`, `user`, `auth`, `tts`). Repository pattern allows future migration from hardcoded data to database or external service without changing domain layer.

## Complexity Tracking

*Not applicable - no constitutional violations*
