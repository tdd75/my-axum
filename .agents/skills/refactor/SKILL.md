---
name: refactor
description: Refactor existing code in the my-axum Rust/Axum workspace without intentionally changing behavior. Use when Codex is asked to clean up, restructure, simplify, deduplicate, rename, move code across existing modules, reduce complexity, improve Rust idioms, or make architecture-preserving changes to APIs, DTOs, use cases, services, repositories, tasks, tests, localization, or shared utilities.
---

# Refactor

## Overview

Use this skill to improve existing implementation quality while preserving runtime behavior and the repository's vertical-slice architecture. Treat refactoring as behavior-sensitive work: understand current contracts, make the smallest coherent change, and prove equivalence with tests.

## Workflow

1. Establish the current behavior before editing.
   - Read `AGENTS.md`.
   - Inspect the touched module, its callers, and matching tests.
   - Use `rg` to find route registrations, DTOs, use cases, services, repositories, i18n keys, task handlers, and test helpers related to the refactor target.
   - Identify public API contracts, database behavior, locale keys, background task semantics, and error surfaces that must stay stable.

2. Define the refactor boundary.
   - Keep changes scoped to the requested code path and directly affected tests.
   - Avoid mixing feature work with refactoring unless the user explicitly asks for a behavior change.
   - Do not change request/response shapes, HTTP status codes, locale key meanings, migration behavior, broker payloads, or persistence semantics accidentally.
   - If a behavior change appears necessary, stop and make it explicit before implementing it.

3. Preserve the architecture.
   - Keep Axum handlers thin: request extraction, context/auth extraction, use case calls, and response mapping only.
   - Keep endpoint workflow and transaction boundaries in `*_use_case.rs`.
   - Keep reusable domain behavior in `*_service.rs` only when it is shared or clearly reusable.
   - Keep SeaORM access and query construction in `*_repository.rs`.
   - Keep API request/response shapes in `*_dto.rs`.
   - Keep background task behavior in `*_task.rs`.
   - Put `context` first when a function accepts a `Context` parameter.

4. Refactor conservatively.
   - Prefer local extraction, renaming, type tightening, error-path simplification, and duplication removal over broad rewrites.
   - Match existing Rust style, module boundaries, naming conventions, and error-handling patterns.
   - Use structured APIs and existing helpers instead of ad hoc parsing or new abstractions.
   - Do not hand-edit generated SeaORM entities in `src/core/db/entity/`.
   - Do not introduce new dependencies unless the refactor clearly cannot be done well with existing tools.

5. Handle user-facing text and tests.
   - Do not introduce hardcoded user-visible strings in Rust source.
   - Preserve existing i18n keys unless renaming is part of the refactor and all usages/locales are updated together.
   - Update tests when names, module locations, helper structure, or expected internal organization changes.
   - Add characterization tests first if the current behavior is under-tested and the refactor could alter behavior.

6. Validate the result.
   - Run the narrowest relevant tests first while iterating.
   - Run `make lint` and `make test-cov` before finishing when feasible.
   - If full validation is blocked or too slow, run the best focused substitute and report exactly what was run and what remains.

## Review Checklist

- Behavior is intentionally unchanged, or each behavior change is called out.
- Public API shapes, status codes, errors, locale meanings, and database semantics are stable.
- The vertical-slice boundaries still match the repository conventions.
- New abstractions remove real duplication or complexity.
- Tests cover the behavior most likely to regress.
- Formatting, Clippy, and coverage commands were run or clearly reported as not run.
