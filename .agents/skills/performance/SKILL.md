---
name: performance
description: Investigate and improve performance in the my-axum Rust/Axum workspace. Use when Codex is asked to profile, optimize latency, throughput, allocations, database queries, pagination, background tasks, broker handling, caching, async concurrency, startup time, benchmark results, or resource usage while preserving API behavior and repository architecture.
---

# Performance

## Overview

Use this skill to make performance work evidence-driven. Measure the current behavior, isolate the bottleneck, apply the smallest useful optimization, and validate that latency, throughput, resource usage, and correctness improve or remain acceptable.

## Workflow

1. Establish the target and baseline.
   - Read `AGENTS.md`.
   - Clarify the performance goal when it is not explicit: latency, throughput, memory, CPU, allocations, query count, startup time, task throughput, or benchmark score.
   - Inspect the related runtime slice, tests, configuration, and any existing benchmark under `benchmark/`.
   - Capture current behavior with the narrowest useful command: focused tests, logs, tracing output, `make benchmark`, database query inspection, or a reproducible local request.
   - Record assumptions when external services such as Postgres, Redis, Kafka, or RabbitMQ are unavailable.

2. Find the bottleneck before editing.
   - Use `rg` to trace handlers, DTOs, use cases, services, repositories, tasks, pagination/order helpers, cache usage, broker usage, and tests.
   - Separate API-layer overhead, use case orchestration, repository queries, serialization, i18n lookup, middleware, task dispatch, and external service calls.
   - Prefer concrete evidence such as query shape, query count, unnecessary clones, repeated allocations, blocking work in async paths, missing pagination limits, N+1 access, excessive transaction scope, or avoidable broker/cache round trips.
   - Do not optimize code just because it looks inefficient unless the change is obviously local and low risk.

3. Choose a scoped optimization.
   - Keep Axum handlers thin and move performance-sensitive workflow changes into use cases, services, or repositories according to the existing architecture.
   - Optimize SeaORM access in repositories: select only needed columns, avoid N+1 queries, preserve ordering and pagination semantics, and keep database-specific behavior explicit.
   - Keep transaction boundaries correct; reduce transaction scope only when it does not weaken consistency.
   - Avoid blocking operations on async executors. Use existing async patterns and task infrastructure instead of ad hoc thread management.
   - Reuse existing caches, pagination helpers, ordering helpers, configuration, and tracing layers before adding new abstractions or dependencies.
   - Do not introduce new services, cache layers, indexes, migrations, or dependencies without clear evidence and tests.

4. Preserve behavior and user-facing contracts.
   - Do not change request/response shapes, status codes, locale key meanings, auth behavior, task semantics, broker payloads, or persistence semantics accidentally.
   - Keep all user-facing strings localized through i18n keys.
   - For schema or index changes, create deterministic SeaORM migrations and add migration tests when relevant.
   - Do not hand-edit generated SeaORM entities in `src/core/db/entity/`; regenerate them when migration changes require it.

5. Test correctness and performance-sensitive behavior.
   - Add or update tests for the behavior most likely to regress.
   - Prefer SQLite-backed tests unless the optimization depends on Postgres-specific SQL, migrations, query plans, or index behavior.
   - Add characterization tests before risky rewrites when current behavior is under-tested.
   - For API optimizations, prefer request/response tests under `tests/<domain>/api/`.
   - For repository optimizations, cover filtering, ordering, pagination, and empty-result behavior.
   - For task optimizations, cover retry/progress/error semantics where practical.

6. Validate and report results.
   - Run focused tests while iterating.
   - Run `make lint` and `make test-cov` before finishing when feasible.
   - Run `make benchmark` or a targeted reproducible measurement when the change claims a performance improvement.
   - Report the before/after signal, commands run, and any measurement caveats.
   - If full validation is blocked by services or time, state the exact blocker and the best substitute validation that was completed.

## Optimization Guardrails

- Prefer measured improvements over speculative rewrites.
- Prefer localized changes over broad architecture churn.
- Keep correctness, security, and observability intact.
- Avoid hiding slow work behind unbounded concurrency, unbounded queues, or cache behavior that can serve stale or unauthorized data.
- Treat migrations, indexes, and cache invalidation as behavior changes that need explicit tests and review.
- Keep performance claims modest when measurements are local, noisy, or incomplete.
