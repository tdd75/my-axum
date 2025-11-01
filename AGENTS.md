# Repository Guidelines

## Project Structure & Module Organization
`my-axum` is a Rust 2024 workspace. The main HTTP app lives in `src/`, shared utilities live in `pkg/`, and SeaORM migrations live in `migration/`. Domain code is organized as vertical slices such as `src/user/` and `src/common/`, with submodules for `api`, `dto`, `use_case`, `service`, `repository`, and `task`.

Core infrastructure lives in `src/core/`:

- `src/core/api/`: router and OpenAPI setup.
- `src/core/async/`: background task types, handlers, cron scheduling, and worker runtime.
- `src/core/db/`: connection, unit of work, pagination, and ordering.
- `src/core/layer/`: Axum/Tower middleware such as auth, CORS, lang, trace, and transaction layers.
- `src/core/runbook/`: runbook registry, shared runbook types, and operational runbook implementations.

Runtime configuration lives in `src/config/`. Keep this area limited to stable configuration, app bootstrap, settings, shutdown, telemetry, and binary entry points under `src/config/bin/`; business workflows should live in domain modules or `src/core/`. Binaries are declared in `Cargo.toml`: the app binary is `my-axum`, with additional `runbook` and `worker` binaries under `src/config/bin/`. Integration and module tests live under `tests/`, mirroring runtime areas. Benchmarks are kept in `benchmark/`.

## Build, Test, and Development Commands
Use the `Makefile` as the primary entry point:

- `make setup`: install `pre-commit`, `sea-orm-cli`, and `cargo-llvm-cov`.
- `make dev`: run the HTTP app with debug logging.
- `make prod`: run the HTTP app in release mode with production-oriented logging.
- `make worker-dev`: run the background worker locally.
- `make worker-prod`: run the background worker in release mode.
- `make test`: run tests for `my-axum` and `pkg`.
- `make test-cov` or `make test-cov-report`: generate coverage summaries or HTML reports.
- `make lint`: format all crates and fail on any Clippy warning.
- `make db-revision name=<migration_name>`: generate a new SeaORM migration.
- `make db-up`, `make db-down`, `make db-seed`: apply migrations, roll back, or seed data.
- `make docker-dev`: start local dependencies from `docker-compose.yml`.
- `make docker-prod`: start the production compose stack.
- `make benchmark`: run the k6 benchmark script.

## Coding Style & Naming Conventions
Follow standard Rust style: 4-space indentation, `snake_case` for files/modules/functions, and `PascalCase` for structs/enums/traits. Keep the existing suffix-based naming scheme:

- `*_api.rs`: Axum handlers, route registration, request extraction, and response mapping.
- `*_dto.rs`: request/response DTOs and API-facing data shapes.
- `*_use_case.rs`: application workflow and transaction boundaries. Every HTTP endpoint must have a corresponding use case entry point; handlers should call the use case rather than orchestrating behavior directly.
- `*_service.rs`: reusable domain behavior only. Put logic here when it can reasonably be shared by multiple use cases; do not use services as endpoint-specific orchestration layers.
- `*_repository.rs`: SeaORM persistence access.
- `*_task.rs`: background task handlers.

Prefer keeping handlers thin. Route functions should parse inputs, call use cases, and map errors/responses. Keep database-specific logic in repositories and transaction-aware or endpoint-specific orchestration in use cases. Use `anyhow` where existing code uses it for internal errors, and keep user/API error surfaces explicit at the API boundary.

Within each `*_api.rs` resource, keep core CRUD handlers first in a consistent order: search, create, read, update, delete. Place non-CRUD endpoints for the same resource after that CRUD block, such as upload, import, export, or other custom actions.

When a function accepts a `Context` parameter alongside other arguments, always put `context` first in the parameter list.

All user-facing strings (API errors, success messages, task progress messages, assistant responses, etc.) must be localized through i18n keys in locale files. Do not hardcode user-visible text directly in Rust source files.

SeaORM entities are maintained by hand in domain modules. Run `make lint` before opening a PR; formatting is enforced through `cargo fmt`, `cargo clippy`, and the local pre-commit hook.

## Testing Guidelines
Every completed feature or behavior change should include tests in the same change. Prefer integration-style tests under `tests/<area>/...` with filenames like `test_user_api.rs`, `test_auth_service.rs`, or `test_create_user_use_case.rs`. Mirror the runtime module layout when adding test modules, and update the corresponding `mod.rs` files. Async tests use `#[tokio::test]`, and reusable bootstrapping belongs in `tests/setup/`.

Keep test cases concise and focused on basic happy-path behavior unless the change specifically requires edge-case or failure coverage. Avoid overbuilding tests for low-risk behavior.

After finishing code changes, run `make test-cov`. Add or update tests so touched lines stay above 90% line coverage before opening a PR.

Default to lightweight SQLite-backed tests when possible. Use Postgres-specific coverage only when the behavior depends on Postgres, migrations, or SQL dialect details. For schema changes, add or update migration tests under `migration/tests/` and run `make test`. For API behavior changes, prefer request/response tests in `tests/<domain>/api/` and include WebSocket coverage when the route is real-time.

Coverage intentionally excludes entry points, migration code, and broker-heavy infrastructure in `Makefile` via `COV_IGNORE`. Do not remove those exclusions unless you are also making those areas testable without external services.

## Commit & Pull Request Guidelines
Git history is minimal, so keep commit messages short, imperative, and descriptive, for example `add refresh token cleanup test`. PRs should include a brief summary, the commands you ran (`make lint`, `make test`, etc.), and notes for any migration, env, or broker-related changes. Link related issues when applicable and include request/response examples when API behavior changes.

## Security & Configuration Tips
Start from `.env.example` and keep secrets out of version control. Use `make docker-dev` to provision PostgreSQL, Redis, Kafka, and RabbitMQ locally. The `Makefile` loads `.env` automatically when present. Keep JWT, SMTP, database, broker, and third-party credentials out of commits and logs.

Migration files should be deterministic, reversible when practical, and named for the schema behavior they introduce.
