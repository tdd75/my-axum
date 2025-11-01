# Axum Boilerplate

A clean and scalable **Axum** project template following the **Layer Architecture** pattern.
This boilerplate is structured for real-world applications with support for authentication, database migrations,
internationalization, and containerized deployment using Rust's high-performance ecosystem.

---

## ✨ Features

- 🚀 **Axum** – Ergonomic and modular web framework built with Tokio
- 🧱 **Layer Architecture** – Modular structure with clear separation of concerns
- 🛢️ **Database Layer** – `SeaORM` with support for PostgreSQL & SQLite + automatic migrations
- 🧪 **Test-Ready** – Built-in testing support with isolated test databases
- 🔐 **Authentication** – JWT-based authentication with Argon2 password hashing
- 🌍 **Internationalization** – Multi-language support using `rust-i18n`
- 📚 **API Documentation** – Auto-generated OpenAPI/Swagger documentation with `utoipa`
- 🎯 **Code Quality** – Formatting and linting via `cargo fmt` & `cargo clippy`
- 📊 **Observability** – Structured logging with `tracing`
- 🐳 **Containerized Deployment** – `Docker` & `Docker Compose` support out of the box
- 📧 **Email Support** – Email functionality with `lettre`
- 🔄 **Message Brokers** – Kafka, RabbitMQ, and Redis support for async task processing
- 🔌 **WebSocket Support** – Real-time bidirectional communication for live updates and synchronization

---

## 🧩 Technologies

<div align="center">
    <code><img width="50" src="https://cdn.simpleicons.org/rust" alt="Rust" title="Rust" /></code>
    <code><img width="50" src="https://www.sea-ql.org/SeaORM/img/SeaQL.png" alt="SeaORM" title="SeaORM" /></code>
    <code><img width="50" src="https://cdn.simpleicons.org/postgresql" alt="PostgreSQL" title="PostgreSQL" /></code>
    <code><img width="50" src="https://cdn.simpleicons.org/sqlite" alt="SQLite" title="SQLite" /></code>
    <code><img width="50" src="https://cdn.simpleicons.org/redis" alt="Redis" title="Redis" /></code>
</div>

<div align="center">
    <code><img width="50" src="https://cdn.simpleicons.org/rabbitmq" alt="RabbitMQ" title="RabbitMQ" /></code>
    <code><img width="50" src="https://cdn.simpleicons.org/apachekafka" alt="Kafka" title="Kafka" /></code>
    <code><img width="50" src="https://cdn.simpleicons.org/docker" alt="Docker" title="Docker" /></code>
    <code><img width="50" src="https://cdn.simpleicons.org/nginx" alt="Nginx" title="Nginx" /></code>
</div>

---

## 📁 Project Structure

```text
my-axum/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── lib.rs                     # Library entry point
│   │
│   ├── config/                    # Configuration layer
│   │
│   ├── core/                      # Core infrastructure
│   │   ├── context.rs             # App context
│   │   ├── db/                    # Database & entities (SeaORM)
│   │   ├── layer/                 # Middleware (auth, CORS, i18n)
│   │   ├── template/              # HTML Templates
│   │   └── translation/           # i18n resources
│   │
│   ├── pkg/                       # Shared packages
│   │
│   └── user/                      # User domain module
│       ├── api/                   # HTTP endpoints
│       ├── dto/                   # Request/Response DTOs
│       ├── use_case/              # Business orchestration
│       ├── service/               # Business logic
│       ├── repository/            # Data access
│       └── task/                  # Background tasks
│
├── migration/                     # Database migrations (SeaORM)
├── tests/                         # Integration & unit tests
├── bin/                           # Utility scripts (seed.rs, worker.rs)
├── benchmark/                     # Performance tests (k6)
└── Makefile                       # Development commands
```

### Architecture Layers

- **API Layer** (`api/`): HTTP endpoints, request validation, routing
- **DTO Layer** (`dto/`): Data transfer objects for serialization
- **Use Case Layer** (`use_case/`): Business flow orchestration
- **Service Layer** (`service/`): Core business logic
- **Repository Layer** (`repository/`): Database operations

Domain modules (e.g., `user/`) are organized as vertical slices containing all necessary layers.

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.9+ (install via [rustup](https://rustup.rs/))
- Docker & Docker Compose
- PostgreSQL (optional if using Docker)

### Setup & Installation

```bash
# Complete setup: install pre-commit hooks, tools, start services, migrate DB, and seed data
make
```

### Development

```bash
# Run application in development mode
make dev

# Run application in production mode
make prod

# Run background worker in development mode
make worker-dev

# Run background worker in production mode
make worker-prod
```

### Docker Services

```bash
# Start development services (PostgreSQL, Redis, Kafka, RabbitMQ)
make docker-dev

# Start production services with Nginx reverse proxy
make docker-prod
```

### Database Operations

```bash
# Create new migration
make db-revision name=create_users_table

# Generate SeaORM entities from database schema
make db-generate

# Apply all pending migrations
make db-up

# Rollback last migration
make db-down

# Seed database with test data
make db-seed
```

### Testing

```bash
# Run all tests
make test

# Run tests with coverage validation
make test-cov

# Generate and open HTML coverage report
make test-cov-report
```

### Code Quality

```bash
# Format code and run linter
make lint
```

### Benchmarking

```bash
# Run k6 performance benchmarks
make benchmark
```

### API Documentation

Once the application is running, visit:

- **Swagger UI**: `http://localhost:8000/docs` - Interactive API documentation
- **Kafka UI**: `http://localhost:8080` - Kafka monitoring and management

---

## 📄 License

Distributed under the [MIT License](./LICENSE). <br>
Feel free to use, modify, and distribute this project.
