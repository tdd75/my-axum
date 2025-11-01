# my-axum

`my-axum` is a Rust workspace built around an Axum HTTP API, a background worker, and a small runbook CLI for operational tasks. The current project includes JWT authentication, user management, SeaORM migrations, OpenAPI docs, message-driven background jobs, and WebSocket endpoints for real-time updates.

Local development defaults to PostgreSQL for application data and Redis as the message broker.

## Current Scope

- Axum HTTP server with routes under `/api/v1/...`
- Swagger UI at `/docs` and OpenAPI JSON at `/docs/openapi.json`
- Auth flows: register, login, refresh token, logout, profile, change password, forgot/reset password
- User search and CRUD endpoints
- Admin-only runbook API and CLI
- Background worker for async tasks
- WebSocket endpoints for user sync and task progress
- SeaORM migrations in a dedicated `migration` crate
- Shared utilities in the `pkg` crate
- Integration tests under `tests/`
- k6 load test script under `benchmark/`

## Workspace Layout

```text
my-axum/
├── src/
│   ├── main.rs                    # HTTP server entrypoint
│   ├── lib.rs
│   ├── common/                    # Shared API/use-case pieces
│   ├── config/                    # App bootstrap, settings, worker, runbooks
│   ├── core/                      # Routing, DB, middleware, DTOs, tasks, templates
│   └── user/                      # Auth, user APIs, use cases, repositories, tasks
├── pkg/                           # Shared crate: jwt, smtp, messaging, cache, url, ...
├── migration/                     # SeaORM migrations
├── tests/                         # Integration and module tests
├── benchmark/                     # k6 benchmark script
├── docker-compose.yml             # Local PostgreSQL + Redis
├── docker-compose.prod.yml        # Production-oriented compose starter
├── Dockerfile
└── Makefile
```

## Local Setup

### Prerequisites

- A recent stable Rust toolchain
- Docker with Docker Compose
- `nc` available locally for `make db-wait`
- `k6` only if you want to run the benchmark script

### Environment

Start from `.env.example`:

```bash
cp .env.example .env
```

Important variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `APP_HOST` | `localhost` | HTTP bind host |
| `APP_PORT` | `8000` | HTTP bind port |
| `DATABASE_URL` | `postgresql://postgres:password@localhost:5432/my_axum` | Main database |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection |
| `MESSAGE_BROKER` | `redis` | Broker for background jobs: `redis`, `kafka`, or `rabbitmq` |
| `JWT_SECRET` | `secret` in `.env.example` | JWT signing secret |
| `SMTP_USER`, `SMTP_PASSWORD` | unset | Required for email delivery tasks |
| `OPENAI_API_KEY` | unset | Required for the authenticated AI assistant endpoint |
| `OPENAI_MODEL` | `gpt-5.4-nano` | OpenAI model used by the assistant |
| `ALLOWED_ORIGINS` | `*` | CORS origins |
| `WORKER_POOL_SIZE` | `10` | Concurrent worker task slots |

If `MESSAGE_BROKER` is unset, the HTTP app can still run, but producer-based flows and the worker will not.

### Quick Start

```bash
cp .env.example .env
make setup
make docker-dev
make db-up
make db-seed
make dev
```

The default `make` target is a first-time bootstrap shortcut and runs:

```bash
make
# equivalent to: make setup docker-dev db-up db-seed
```

Once the app is running:

- API: `http://localhost:8000`
- Swagger UI: `http://localhost:8000/docs`
- OpenAPI JSON: `http://localhost:8000/docs/openapi.json`

Run the worker in a second terminal when you want async tasks to be consumed:

```bash
make worker-dev
```

### Seed Data

`make db-seed` runs the `seed` runbook and creates these default accounts if they do not already exist:

- `admin@example.com` / `admin123@`
- `user@example.com` / `password123@`

The seed runbook is idempotent, so re-running it does not duplicate users.

## Common Commands

| Command | Description |
| --- | --- |
| `make setup` | Install `pre-commit`, `sea-orm-cli`, and `cargo-llvm-cov` |
| `make dev` | Run the HTTP server with debug logging |
| `make prod` | Run the HTTP server in release mode |
| `make worker-dev` | Run the background worker locally |
| `make worker-prod` | Run the background worker in release mode |
| `make test` | Run tests across the entire Rust workspace |
| `make test-cov` | Print workspace coverage summary |
| `make test-cov-report` | Generate and open the workspace HTML coverage report |
| `make lint` | Run `cargo fmt` and fail on any Clippy warning |
| `make db-revision name=...` | Generate a new SeaORM migration |
| `make db-up` | Apply migrations |
| `make db-down` | Roll back the latest migration |
| `make db-generate` | Regenerate SeaORM entities into `src/core/db/entity/` |
| `make db-seed` | Run the seed runbook |
| `make docker-dev` | Start local PostgreSQL and Redis containers |
| `make benchmark` | Run the k6 benchmark script |

## HTTP API

Refer to the Swagger UI at `/docs` or the OpenAPI JSON at `/docs/openapi.json` for a complete and up-to-date list of available endpoints and their requirements.

Authenticated HTTP routes use `Authorization: Bearer <access_token>`.

### WebSocket Endpoints

WebSocket authentication is passed as a query parameter (e.g., `?token=<access_token>`).
For locale-aware task updates, clients can also pass `?lang=<locale>` on the websocket URL.

## Runbook CLI and API

The runbook binary is intended for operational scripts that reuse application services and data access.

List available runbooks:

```bash
cargo run --bin runbook -- list
```

Run the seed script:

```bash
cargo run --bin runbook -- run seed
```

Delete refresh tokens for a user:

```bash
cargo run --bin runbook -- run delete-refresh-tokens-by-email --email user@example.com
```

The same functionality is exposed over HTTP for authenticated admin users:

```bash
curl http://localhost:8000/api/v1/runbook/ \
  -H "Authorization: Bearer <admin_access_token>"
```

```bash
curl -X POST http://localhost:8000/api/v1/runbook/run/ \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"delete-refresh-tokens-by-email","args":["--email","user@example.com"]}'
```

## Background Worker

The `worker` binary consumes background jobs from the configured broker and also initializes scheduled job publishing.

Notes:

- Local `docker-compose.yml` starts PostgreSQL and Redis only.
- Kafka and RabbitMQ adapters exist in the codebase, but their compose services are kept commented out by default.
- Email delivery requires valid SMTP credentials in the environment.

## Testing and Benchmarking

- `make test` runs `cargo test --workspace`
- This includes the crates in the current workspace, not just `my-axum`
- The test suite prefers lightweight SQLite-backed execution where possible
- Use PostgreSQL-backed tests only when behavior depends on the real database engine
- `make test-cov` and `make test-cov-report` use workspace coverage with the exclusions configured in `Makefile`
- `make benchmark` runs the k6 script in `benchmark/index.js`

The benchmark script expects the API to be running and uses the seeded admin account by default.

## Deployment Notes

The repo includes a `Dockerfile`, `docker-compose.prod.yml`, and `nginx.conf`. Treat them as a deployment starting point rather than the default local path. Review environment variables, bootstrap commands, and enabled broker services before using them in production.

## License

Distributed under the [MIT License](./LICENSE).
