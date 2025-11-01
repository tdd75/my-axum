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

# Exclude entry points and infrastructure code from coverage:
# - main.rs, config/cmd/bin/*.rs: application entry points
# - cron.rs: scheduler infrastructure
# - cmd/worker.rs: worker entry point logic
# - core/db/entity/: generated SeaORM entities
# - messaging/consumer, messaging/producer: requires message broker infrastructure
# - broadcast/forwarder: requires message broker infrastructure
COV_IGNORE := --ignore-filename-regex='(main\.rs|config/cmd/bin/runbook\.rs|config/cmd/bin/worker\.rs|config/cron\.rs|config/cmd/worker\.rs|core/db/entity/|messaging/consumer|messaging/producer|broadcast/forwarder|messaging/util)'

ifdef COV
COV_FLAG := --fail-under-lines $(COV)
endif

.PHONY: test
test:
	RUST_BACKTRACE=1 cargo test --workspace

.PHONY: test-cov
test-cov:
	cargo llvm-cov --workspace --no-cfg-coverage --summary-only $(COV_FLAG) $(COV_IGNORE)

.PHONY: test-cov-report
test-cov-report:
	cargo llvm-cov --workspace --no-cfg-coverage --html --open $(COV_FLAG) $(COV_IGNORE)

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
	@set -e; \
	branch=$$(git branch --show-current); \
	if [ -z "$$branch" ]; then \
		echo "Not on a branch; refusing to amend and force-push."; \
		exit 1; \
	fi; \
	echo "Current branch: $$branch"; \
	echo "Amending commit: $$(git log -1 --oneline)"; \
	echo "Current changes:"; \
	git status --short; \
	attempt=1; \
	max_attempts=3; \
	while [ $$attempt -le $$max_attempts ]; do \
		echo "Staging changes..."; \
		git add .; \
		echo "Running git commit --amend --no-edit (attempt $$attempt/$$max_attempts)..."; \
		if git commit --amend --no-edit; then \
			if [ -z "$$(git status --short)" ]; then \
				break; \
			fi; \
			echo "Commit succeeded, but hooks modified files. Re-amending..."; \
		else \
			if [ -z "$$(git status --short)" ] || [ $$attempt -eq $$max_attempts ]; then \
				echo "Commit amend failed. Fix the reported error and rerun make commit-amend."; \
				exit 1; \
			fi; \
			echo "Commit amend failed and files changed, likely from lint/pre-commit hooks. Re-staging and retrying..."; \
		fi; \
		attempt=$$((attempt + 1)); \
	done; \
	if [ $$attempt -gt $$max_attempts ]; then \
		echo "Commit amend did not stabilize after $$max_attempts attempts."; \
		exit 1; \
	fi; \
	echo "Pushing amended commit with --force-with-lease..."; \
	git push --force-with-lease

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

.PHONY: db-up
db-up: db-wait
	cargo run -p migration -- up

.PHONY: db-down
db-down: db-wait
	cargo run -p migration -- down

.PHONY: db-generate
db-generate: db-wait
	sea-orm-cli generate entity --output-dir src/core/db/entity --entity-format dense

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
