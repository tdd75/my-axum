---
name: test
description: Run coverage validation and raise line coverage in the my-axum Rust/Axum workspace. Use when Codex is asked to run make test-cov, inspect coverage output, add or update tests when line coverage is below 90%, and rerun coverage until the target is met or a blocker is identified.
---

# Test

## Overview

Use this skill to make coverage work actionable. Run `make test-cov`, read the line coverage result, add focused tests for uncovered behavior when line coverage is below 90%, and rerun coverage validation.

## Workflow

1. Run coverage first.
   - Read `AGENTS.md`.
   - Run `make test-cov` from the repository root.
   - Capture the overall line coverage and any per-file or per-module coverage details shown in the output.
   - If the command fails before reporting coverage, fix or report the test/build failure before writing new coverage tests.

2. Decide whether tests are needed.
   - If line coverage is 90% or higher, report the result and stop.
   - If line coverage is below 90%, identify the files or behavior contributing most to the gap.
   - Prefer tests for recently touched code, risky behavior, and uncovered branches that represent real user or domain behavior.
   - Do not add shallow tests that only execute code without checking meaningful outcomes.

3. Add focused tests.
   - Prefer integration-style tests under `tests/<area>/...`, mirroring runtime module layout.
   - Use filenames like `test_user_api.rs`, `test_auth_service.rs`, or `test_create_user_use_case.rs`.
   - Update the corresponding `mod.rs` files so new tests compile.
   - Use `#[tokio::test]` for async tests.
   - Default to SQLite-backed tests when possible.
   - Use Postgres-specific tests only when the behavior depends on migrations, SQL dialect details, or Postgres query behavior.
   - For API behavior, prefer request/response tests under `tests/<domain>/api/`.
   - For repositories, cover filters, ordering, pagination, empty results, missing rows, and error cases.
   - For use cases and services, cover success paths, validation failures, permission failures, and edge cases.
   - For schema changes, add or update migration tests under `migration/tests/`.

4. Keep behavior and architecture intact.
   - Do not change production code just to raise coverage unless the test exposes a real bug or missing seam already consistent with the architecture.
   - Keep user-facing strings localized through i18n keys.
   - Do not remove or weaken existing tests.
   - Do not remove `COV_IGNORE` exclusions from `Makefile` unless also making those excluded areas testable without external services.

5. Rerun validation.
   - Run the narrowest new or changed tests first while iterating.
   - Rerun `make test-cov` after adding tests.
   - Repeat the add-test and rerun cycle until line coverage is above 90% or a concrete blocker remains.
   - Report final line coverage, tests added, commands run, and any remaining uncovered risk.

## Guardrails

- Prioritize meaningful assertions over coverage-only execution.
- Avoid tests that depend on execution order, wall-clock timing, real external services, or shared mutable state unless the test setup isolates them.
- Keep fixtures and setup reusable under `tests/setup/` when multiple tests need the same bootstrapping.
- If coverage output is noisy or truncated, rerun with the narrowest available coverage/report command and inspect the generated report when available.
