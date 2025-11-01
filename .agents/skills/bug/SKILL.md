---
name: bug
description: Diagnose and fix bugs in the my-axum Rust/Axum workspace. Use when Codex is asked to reproduce failures, investigate incorrect API behavior, failing tests, regressions, panics, validation errors, database/query bugs, i18n issues, auth or middleware defects, background task failures, broker/cache problems, or production-like defects while preserving intended behavior and repository architecture.
---

# Bug

## Overview

Use this skill to fix defects by proving the failure, finding the root cause, making the smallest correct change, and adding regression coverage. Prefer evidence over guesses, and keep the fix inside the repository's existing vertical-slice boundaries.

## Workflow

1. Reproduce and define the bug.
   - Read `AGENTS.md`.
   - Capture the reported symptom, expected behavior, actual behavior, inputs, environment, and affected route/task/module.
   - Run the failing test or the narrowest command that reproduces the problem.
   - If there is no existing reproduction, add or draft a focused regression test before changing implementation when practical.
   - Record external dependencies that affect reproduction, such as Postgres, Redis, Kafka, RabbitMQ, SMTP, or environment variables.

2. Trace the affected path.
   - Use `rg` to find related routes, DTOs, use cases, services, repositories, tasks, middleware, locale keys, configuration, and tests.
   - Follow the request or task flow through handlers, context/auth extraction, use cases, repositories/services, and response/error mapping.
   - Inspect persistence behavior, transaction boundaries, pagination/order helpers, cache keys, broker payloads, and generated entities only as needed.
   - Separate root cause from incidental cleanup opportunities.

3. Choose the smallest correct fix.
   - Keep Axum handlers thin: extraction, use case call, and response mapping only.
   - Put endpoint workflow fixes in `*_use_case.rs`.
   - Put reusable domain fixes in `*_service.rs` only when shared behavior is actually affected.
   - Put SeaORM query and persistence fixes in `*_repository.rs`.
   - Put request/response shape fixes in `*_dto.rs`.
   - Put background task fixes in `*_task.rs`.
   - Put `context` first when a function accepts a `Context` parameter.

4. Preserve contracts while fixing.
   - Do not change API shapes, status codes, locale key meanings, auth behavior, task semantics, broker payloads, or database semantics unless the bug is precisely that contract.
   - Keep all user-facing strings localized through i18n keys.
   - Do not hand-edit generated SeaORM entities in `src/core/db/entity/`.
   - If a schema fix is required, create a deterministic SeaORM migration and add or update migration tests.
   - Avoid broad refactors while fixing a bug; leave unrelated cleanup for a separate change.

5. Add regression coverage.
   - Add or update the smallest test that fails before the fix and passes after it.
   - Prefer integration-style tests under `tests/<area>/...`, mirroring runtime layout.
   - Use SQLite-backed tests unless the bug depends on Postgres-specific SQL, migrations, or dialect behavior.
   - For API bugs, prefer request/response tests under `tests/<domain>/api/`.
   - For repository bugs, cover filtering, ordering, pagination, missing rows, and edge cases relevant to the defect.
   - For task bugs, cover retry, progress, failure, and side-effect semantics where practical.

6. Validate and explain.
   - Run the focused regression test first.
   - Run nearby tests for the touched module.
   - Run `make lint` and `make test-cov` before finishing when feasible.
   - Explain the root cause, the fix, and the exact commands run.
   - If full validation is blocked or too slow, state what was run and what remains.

## Bug-Fix Guardrails

- Do not fix symptoms by weakening validation, suppressing errors, or broadening catch-all handling without proving it is correct.
- Do not hide failures behind default values unless the default is part of the intended contract.
- Do not remove tests to make a failure pass.
- Keep security-sensitive behavior strict for auth, JWT, permissions, secrets, request validation, and cross-user data access.
- Treat race conditions, transaction changes, cache invalidation, and broker retry behavior as high-risk and test them explicitly where practical.
