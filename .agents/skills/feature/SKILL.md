---
name: feature
description: Add or change product features in the my-axum Rust/Axum workspace. Use when Codex is asked to implement a new API endpoint, domain workflow, repository operation, background task, DTO, validation rule, localization-backed user message, database-backed behavior, migration-related feature, or tests for feature behavior in this repository.
---

# Feature

## Overview

Use this skill to add feature work end to end while preserving the repository's vertical-slice architecture and testing expectations. Keep handlers thin, put orchestration in use cases, persistence in repositories, and all user-facing strings behind i18n keys.

## Workflow

1. Inspect the existing slice before editing.
   - Read `AGENTS.md`.
   - Find the closest domain under `src/<domain>/` and matching tests under `tests/<domain>/`.
   - Search existing APIs, DTOs, use cases, services, repositories, locale keys, and tests with `rg`.
   - Check whether the feature needs schema changes, generated entities, background tasks, or broker/external dependencies.

2. Choose the implementation shape.
   - Add or update `*_api.rs` only for route wiring, request extraction, auth/context extraction, and response mapping.
   - Add or update `*_use_case.rs` for endpoint-specific workflow, transaction boundaries, and calls across repositories/services.
   - Add or update `*_service.rs` only for reusable domain behavior shared by multiple use cases.
   - Add or update `*_repository.rs` for SeaORM queries and persistence.
   - Add or update `*_dto.rs` for API request/response shapes.
   - Put `context` first when a function accepts a `Context` parameter.

3. Handle localization.
   - Do not add hardcoded user-facing strings in Rust source.
   - Add i18n keys in the relevant locale files for API errors, success messages, task progress, assistant responses, and similar user-visible text.
   - Reuse nearby naming patterns for locale keys.

4. Handle data changes conservatively.
   - If a schema change is required, create a SeaORM migration with `make db-revision name=<migration_name>`.
   - Do not hand-edit `src/core/db/entity/` unless intentionally reviewing generated output.
   - After migrations, use `make db-generate` when entities need regeneration and review generated diffs carefully.

5. Add tests with the feature.
   - Prefer integration-style tests under `tests/<domain>/...`, mirroring runtime layout.
   - Use `#[tokio::test]` for async tests.
   - Prefer SQLite-backed tests unless behavior depends on Postgres, migrations, or SQL dialect details.
   - Update the relevant `mod.rs` files so new tests compile.
   - For API changes, prefer request/response tests in `tests/<domain>/api/`.
   - For schema changes, add or update migration tests under `migration/tests/`.

6. Validate.
   - Run the narrowest useful tests first while iterating.
   - Before finishing, run `make lint` and `make test-cov` when feasible.
   - If full coverage validation is too slow or blocked by missing services, report exactly what was run and what remains.

## Implementation Notes

- Preserve existing suffix-based file naming: `*_api.rs`, `*_dto.rs`, `*_use_case.rs`, `*_service.rs`, `*_repository.rs`, and `*_task.rs`.
- Keep database-specific logic out of handlers and use cases when it belongs in repositories.
- Use `anyhow` only where the existing code uses it for internal errors; keep API error surfaces explicit at the boundary.
- Avoid broad refactors while adding a feature. Touch only the files required for the behavior, tests, and localization.
- Never remove coverage exclusions from `Makefile` unless also making those excluded areas testable without external services.
