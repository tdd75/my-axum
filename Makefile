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

COVERAGE_IGNORE := --ignore-filename-regex='main\.rs|migration/|bin/|config/|pkg/messaging/|pkg/broadcast/|core/db/entity/'

.PHONY: test-db-cleanup
test-db-cleanup:
	rm -rf ./data/sqlite

.PHONY: test
test: test-db-cleanup
	RUST_BACKTRACE=1 cargo test --workspace

.PHONY: test-cov
test-cov: test-db-cleanup
	cargo llvm-cov --workspace --summary-only --fail-under-functions 80 --fail-under-lines 90 --fail-under-regions 80 $(COVERAGE_IGNORE)

.PHONY: test-cov-report
test-cov-report: test-db-cleanup
	cargo llvm-cov --workspace --html --no-cfg-coverage --open $(COVERAGE_IGNORE)

# ------------------------------------------------------------------------------
# Formatting & Linting
# ------------------------------------------------------------------------------

.PHONY: lint
lint:
	cargo fmt --all -- --color always
	cargo clippy --all-targets --all-features -- -D warnings

# ------------------------------------------------------------------------------
# Database
# ------------------------------------------------------------------------------

.PHONY: db-wait
db-wait:
	@echo "Waiting for PostgreSQL to be ready..."
	@DB_HOST=$$(echo $(DATABASE_URL) | sed -E 's|postgresql(\+[^:]+)?://[^@]*@([^:/]+).*|\2|'); \
	DB_PORT=$$(echo $(DATABASE_URL) | sed -E 's|.*:([0-9]+)/.*|\1|' | grep -E '^[0-9]+$$' || echo "5432"); \
	until nc -z -w 1 $$DB_HOST $$DB_PORT > /dev/null 2>&1; do \
		echo "PostgreSQL is unavailable - sleeping"; \
		sleep 1; \
	done;
	@echo "Port is open, waiting for app layer to stabilize..."
	@sleep 2
	@echo "PostgreSQL is up and ready!"

.PHONY: db-revision
db-revision: db-wait
	sea-orm-cli migrate generate ${name}

.PHONY: db-generate
db-generate: db-wait
	sea-orm-cli generate entity --output-dir src/core/db/entity --entity-format dense

.PHONY: db-up
db-up: db-wait
	sea-orm-cli migrate up

.PHONY: db-down
db-down: db-wait
	sea-orm-cli migrate down

.PHONY: db-seed
db-seed: db-wait
	cargo run --bin seed

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
