# Load environment variables from .env file if it exists
ifneq (,$(wildcard ./.env))
    include .env
    export
endif

# Default target
.PHONY: all
all: setup docker-dev db-up db-seed

# ------------------------------------------------------------------------------
# Environment Setup
# ------------------------------------------------------------------------------

.PHONY: setup
setup:
	pre-commit install
	cargo install sea-orm-cli@^2.0.0-rc cargo-llvm-cov

# ------------------------------------------------------------------------------
# Running the Application
# ------------------------------------------------------------------------------

.PHONY: dev
dev:
	RUST_LOG=debug,sqlx::query=warn,rdkafka=info RUST_BACKTRACE=1 cargo run --bin my-axum

.PHONY: prod
prod:
	RUST_LOG=info,sqlx::query=warn,rdkafka=warn APP_HOST=0.0.0.0 cargo run --release --bin my-axum

.PHONY: worker-dev
worker-dev:
	RUST_LOG=debug,sqlx::query=warn,rdkafka=info cargo run --bin worker

.PHONY: worker-prod
worker-prod:
	RUST_LOG=info,sqlx::query=warn,rdkafka=warn cargo run --release --bin worker

# ------------------------------------------------------------------------------
# Testing
# ------------------------------------------------------------------------------

TEST_PACKAGES := -p my-axum -p pkg

# Exclude entry points and infrastructure code from coverage:
# - main.rs, config/bin/: application entry points
# - core/async/: scheduler and worker infrastructure
# - messaging/, broadcast/: requires message broker infrastructure
COV_IGNORE := --ignore-filename-regex='(main\.rs|config/bin/|core/async/|messaging/|broadcast/)'

ifdef COV
COV_FLAG := --fail-under-lines $(COV)
endif

.PHONY: test
test:
	RUST_BACKTRACE=1 cargo test $(TEST_PACKAGES)

.PHONY: test-cov
test-cov:
	cargo llvm-cov $(TEST_PACKAGES) --no-cfg-coverage --summary-only $(COV_FLAG) $(COV_IGNORE)

.PHONY: test-cov-report
test-cov-report:
	cargo llvm-cov $(TEST_PACKAGES) --no-cfg-coverage --html --open $(COV_FLAG) $(COV_IGNORE)

# ------------------------------------------------------------------------------
# Formatting & Linting
# ------------------------------------------------------------------------------

.PHONY: lint
lint:
	cargo fmt --all -- --color always
	cargo clippy --all-targets --all-features -- -D warnings

# ------------------------------------------------------------------------------
# Git
# ------------------------------------------------------------------------------

.PHONY: commit-amend
commit-amend:
	./scripts/commit-amend.sh

# ------------------------------------------------------------------------------
# Database
# ------------------------------------------------------------------------------

.PHONY: db-wait
db-wait:
	./scripts/db-wait.sh

.PHONY: db-revision
db-revision: db-wait
	sea-orm-cli migrate generate ${name}

.PHONY: db-up
db-up: db-wait
	cargo run -p migration -- up

.PHONY: db-down
db-down: db-wait
	cargo run -p migration -- down

.PHONY: db-seed
db-seed: db-wait
	cargo run --bin runbook -- run seed

# ------------------------------------------------------------------------------
# Docker
# ------------------------------------------------------------------------------

.PHONY: docker-dev
docker-dev:
	docker compose down
	docker compose up --build -d

.PHONY: docker-prod
docker-prod:
	docker compose -f docker-compose.prod.yml down
	docker compose -f docker-compose.prod.yml up --build -d

# ------------------------------------------------------------------------------
# Benchmarking
# ------------------------------------------------------------------------------

.PHONY: benchmark
benchmark:
	k6 run benchmark/index.js
