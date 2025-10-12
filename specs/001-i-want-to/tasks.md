/speckit.implement---
description: "Task list for feed suggestions by category feature"
---

# Tasks: Feed Suggestions by Category

**Input**: Design documents from `/specs/001-i-want-to/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/openapi.yaml, research.md

**Tests**: E2E tests following constitutional requirement IV (Test-First with Real Infrastructure)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root (this project)
- Paths follow Rust conventions with domain-driven architecture

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 [P] Create `src/domain/feed_suggestions/` directory for new domain module
- [X] T002 [P] Create `src/infrastructure/repositories/` files if not exists (already exists, verify structure)
- [X] T003 [P] Create `tests/e2e/` files if not exists (already exists, verify structure)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core domain entities and repository trait that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 [P] [FOUNDATION] Define `Category` entity in `src/domain/feed_suggestions/mod.rs` with id, name, description fields (serde Serialize/Deserialize)
- [X] T005 [P] [FOUNDATION] Define `FeedSuggestion` entity in `src/domain/feed_suggestions/mod.rs` with id, title, description, url, category_id fields (serde)
- [X] T006 [FOUNDATION] Define `FeedSuggestionsRepository` trait in `src/domain/feed_suggestions/mod.rs` with `get_all_categories()` and `get_suggestions_by_categories()` methods (depends on T004, T005)
- [X] T007 [FOUNDATION] Register `feed_suggestions` module in `src/domain/mod.rs`

**Checkpoint**: Foundation ready - domain entities and repository trait defined

---

## Phase 3: User Story 1 - New User Onboarding with Category Selection (Priority: P1) 🎯 MVP

**Goal**: Enable new users to discover curated feeds by selecting categories of interest

**Independent Test**: Create a new user account, retrieve categories, filter suggestions by category, verify 4 suggestions per category returned with correct structure

### Implementation for User Story 1

- [X] T008 [P] [US1] Create `FeedSuggestionsService` struct in `src/domain/feed_suggestions/service.rs` with repository dependency
- [X] T009 [P] [US1] Implement `get_suggestions(&self, category_ids: Vec<String>) -> Vec<FeedSuggestion>` method in service with empty category_ids handling
- [X] T010 [US1] Implement `HardcodedFeedSuggestionsRepository` struct in `src/infrastructure/repositories/feed_suggestions_repository.rs` (new file)
- [X] T011 [US1] Create `CATEGORIES` static LazyLock with all 20 categories in repository (news-current-affairs, technology-programming, science-research, business-finance, design-creativity, gaming-entertainment, health-fitness, food-cooking, travel-adventure, books-literature, movies-tv, music-podcasts, sports, environment-sustainability, politics-policy, personal-development, lifestyle-home, automotive, fashion-beauty, education-learning)
- [X] T012 [US1] Create `FEED_SUGGESTIONS` static LazyLock with all 80 feed suggestions (4 per category) in repository using user-provided feed list
- [X] T013 [US1] Implement `FeedSuggestionsRepository` trait for `HardcodedFeedSuggestionsRepository` with `get_suggestions_by_categories()` using HashSet deduplication by URL
- [X] T014 [US1] Add debug assertions in repository constructor to verify 20 categories and 80 suggestions with exactly 4 per category
- [X] T015 [US1] Register repository module in `src/infrastructure/repositories/mod.rs`
- [X] T016 [P] [US1] Create request/response DTOs in `src/controllers/feed_suggestions.rs`: `GetSuggestionsQuery`, `SuggestionsResponse`, `FeedSuggestionResponse`, `CategoryRef`
- [X] T017 [US1] Implement `get_suggestions` handler in `src/controllers/feed_suggestions.rs` that parses query params (category_ids or categories alias), calls service, transforms to response with nested category info (depends on T016)
- [X] T018 [US1] Register `feed_suggestions` controller module in `src/controllers/mod.rs`
- [X] T019 [US1] Register `/api/feed-suggestions` GET route in `src/infrastructure/http/mod.rs` with auth middleware and service state
- [X] T020 [US1] Wire up `FeedSuggestionsService` in `src/main.rs`: create repository, create service with Arc, pass to router
- [X] T021 [P] [US1] Add logging for category filter requests in service with tracing::info
- [X] T022 [P] [US1] Add warning logs for invalid category IDs in repository deduplication logic with tracing::warn

### E2E Tests for User Story 1

- [X] T023 [P] [US1] Create `tests/e2e/test_feed_suggestions.rs` file
- [X] T024 [P] [US1] Write E2E test `it_should_get_suggestions_by_single_category` testing technology-programming returns 4 suggestions
- [X] T025 [P] [US1] Write E2E test `it_should_get_suggestions_by_multiple_categories` testing 2 categories returns 8 suggestions (4+4)
- [X] T026 [P] [US1] Write E2E test `it_should_deduplicate_suggestions_across_categories` verifying no duplicate URLs in response
- [X] T027 [P] [US1] Write E2E test `it_should_return_empty_for_invalid_category_ids` verifying empty suggestions array
- [X] T028 [P] [US1] Write E2E test `it_should_require_authentication` verifying 401 without JWT token

**Checkpoint**: User Story 1 complete - new users can filter feed suggestions by category

---

## Phase 4: User Story 2 - Browse All Available Categories (Priority: P2)

**Goal**: Allow users to view all available categories with descriptions before selecting

**Independent Test**: Call categories endpoint, verify 20 categories returned with id, name, description in alphabetical order

### Implementation for User Story 2

- [X] T029 [US2] Implement `get_categories(&self) -> Vec<Category>` method in `FeedSuggestionsService` at `src/domain/feed_suggestions/service.rs`
- [X] T030 [US2] Implement `get_all_categories()` method in `HardcodedFeedSuggestionsRepository` returning cloned CATEGORIES Vec sorted alphabetically by name
- [X] T031 [P] [US2] Create `CategoriesResponse` DTO in `src/controllers/feed_suggestions.rs`
- [X] T032 [US2] Implement `get_categories` handler in `src/controllers/feed_suggestions.rs` calling service and returning JSON response (depends on T031)
- [X] T033 [US2] Register `/api/feed-suggestions/categories` GET route in `src/infrastructure/http/mod.rs` with auth middleware

### E2E Tests for User Story 2

- [X] T034 [P] [US2] Write E2E test `it_should_get_all_categories` verifying exactly 20 categories returned
- [X] T035 [P] [US2] Write E2E test `it_should_return_categories_with_all_fields` verifying id, name, description present in each category
- [X] T036 [P] [US2] Write E2E test `it_should_sort_categories_alphabetically` verifying categories ordered by name

**Checkpoint**: User Story 2 complete - users can browse all categories

---

## Phase 5: User Story 3 - Refresh Feed Suggestions (Priority: P3)

**Goal**: Enable existing users to browse suggestions without forced onboarding flow

**Independent Test**: Use existing user account (with feeds already added), access suggestions endpoint, verify suggestions returned without errors

### Implementation for User Story 3

**Note**: No new implementation needed - existing endpoints support this use case. This phase validates the behavior.

### E2E Tests for User Story 3

- [X] T037 [US3] Write E2E test `it_should_allow_existing_users_to_browse_suggestions` creating user, adding feeds, then accessing suggestions endpoint
- [X] T038 [US3] Write E2E test `it_should_return_consistent_suggestions_for_same_category` calling endpoint twice for same category and comparing results

**Checkpoint**: User Story 3 complete - existing users can refresh/browse suggestions

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T039 [P] Run `cargo check` to verify compilation
- [X] T040 [P] Run `cargo clippy` to check linter warnings
- [X] T041 [P] Run `cargo fmt` to format code
- [X] T042 Run `cargo test` to execute all E2E tests
- [X] T043 [P] Verify response times under 500ms by adding timing logs or running manual performance test
- [ ] T044 [P] Update `CLAUDE.md` if any new patterns or conventions introduced (optional)
- [ ] T045 Validate quickstart.md instructions by following step-by-step (optional)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3, 4, 5)**: All depend on Foundational phase completion
  - User Story 1 (Phase 3): Can start after Foundational - No dependencies on other stories
  - User Story 2 (Phase 4): Can start after Foundational - No dependencies on US1 (independent)
  - User Story 3 (Phase 5): Depends on US1 being complete (uses same endpoints)
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independently testable
- **User Story 3 (P3)**: Depends on User Story 1 (validates existing behavior)

### Within Each User Story

**User Story 1 (Core Suggestions)**:
1. Service methods (T008, T009) in parallel
2. Repository implementation (T010-T015) sequential (data depends on structs)
3. Controller DTOs and handlers (T016-T018) sequential (handler depends on DTOs)
4. Route registration (T019) after controller
5. Main wiring (T020) after all components
6. Logging (T021-T022) in parallel with other tasks
7. E2E tests (T023-T028) all in parallel after implementation

**User Story 2 (Categories)**:
1. Service method (T029) and repository method (T030) sequential
2. Controller DTO and handler (T031-T032) sequential
3. Route registration (T033) after handler
4. E2E tests (T034-T036) all in parallel after implementation

**User Story 3 (Refresh)**:
1. E2E tests (T037-T038) in parallel - no new implementation needed

### Parallel Opportunities

- **Setup tasks (Phase 1)**: All tasks marked [P] can run in parallel
- **Foundational tasks (Phase 2)**: T004 and T005 in parallel, T006 after both, T007 after T006
- **User Story 1**: T008 and T009 in parallel, T016 independent, T021 and T022 in parallel, all E2E tests in parallel
- **User Story 2**: T031 independent, all E2E tests in parallel
- **User Story 3**: Both E2E tests in parallel
- **Once Foundational completes**: US1 and US2 can be worked in parallel by different developers

---

## Parallel Example: User Story 1

```bash
# After Foundational phase, launch these tasks in parallel:
Task: "Create FeedSuggestionsService struct"
Task: "Implement get_suggestions method"
Task: "Create request/response DTOs"

# After repository complete, launch these in parallel:
Task: "Add logging for category filter requests"
Task: "Add warning logs for invalid category IDs"

# After implementation complete, launch all E2E tests in parallel:
Task: "E2E test for single category"
Task: "E2E test for multiple categories"
Task: "E2E test for deduplication"
Task: "E2E test for invalid IDs"
Task: "E2E test for authentication"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (T008-T028)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Run E2E tests to verify all scenarios pass
6. Deploy/demo if ready - this is the complete MVP!

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP! 🎯)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (T008-T028)
   - Developer B: User Story 2 (T029-T036) - can start in parallel with US1
   - Developer C: Polish tasks or wait for US1 to complete before US3
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All 80 feed suggestions must be added in T012 (see user-provided feed list)
- Repository uses LazyLock (Rust 1.70+) or lazy_static for older versions
- Deduplication uses HashSet with URL as unique key
- Invalid category IDs are filtered silently with warning logs, not errors
- Both endpoints require JWT authentication (existing middleware)
- Response times must be under 500ms (verify in T043)

---

## Task Count Summary

**Total Tasks**: 45
- Phase 1 (Setup): 3 tasks
- Phase 2 (Foundational): 4 tasks
- Phase 3 (User Story 1): 21 tasks (15 implementation + 6 E2E tests)
- Phase 4 (User Story 2): 8 tasks (5 implementation + 3 E2E tests)
- Phase 5 (User Story 3): 2 tasks (2 E2E tests)
- Phase 6 (Polish): 7 tasks

**Parallel Opportunities**: 18 tasks marked [P] for parallel execution

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 = 28 tasks for complete MVP
