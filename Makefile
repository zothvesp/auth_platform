# AuthForge Makefile
# Usage: make <target>
# Run `make help` to see all targets.

.PHONY: help keys dev dev-backend dev-frontend stop kill-ports build build-backend build-frontend \
        test test-backend test-frontend test-frontend-e2e lint lint-backend lint-frontend \
        db-up db-down db-migrate db-reset seed clean \
        audit audit-backend audit-frontend

# ─── Colours ──────────────────────────────────────────────────────────────────
BOLD  := \033[1m
RESET := \033[0m
GREEN := \033[32m
CYAN  := \033[36m

# ─── Paths ────────────────────────────────────────────────────────────────────
BACKEND_DIR  := backend
FRONTEND_DIR := frontend
KEYS_DIR     := backend/keys

# ─── Default ──────────────────────────────────────────────────────────────────
.DEFAULT_GOAL := help

help: ## Show this help
	@echo ""
	@echo "  $(BOLD)AuthForge$(RESET) — available targets:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""

# ─── Setup ────────────────────────────────────────────────────────────────────

keys: ## Generate RSA-2048 key pair and vault master key (run once)
	@mkdir -p $(KEYS_DIR)
	@if [ -f $(KEYS_DIR)/private.pem ]; then \
		echo "  Keys already exist — delete $(KEYS_DIR)/ to regenerate."; \
	else \
		openssl genrsa -out $(KEYS_DIR)/private.pem 2048 2>/dev/null && \
		openssl rsa -in $(KEYS_DIR)/private.pem -pubout -out $(KEYS_DIR)/public.pem 2>/dev/null && \
		echo "  $(GREEN)✓$(RESET) Generated $(KEYS_DIR)/private.pem and $(KEYS_DIR)/public.pem"; \
	fi
	@if ! grep -q "^VAULT_MASTER_KEY=" $(BACKEND_DIR)/.env 2>/dev/null; then \
		VAULT_KEY=$$(openssl rand -hex 32) && \
		echo "VAULT_MASTER_KEY=$$VAULT_KEY" >> $(BACKEND_DIR)/.env && \
		echo "  $(GREEN)✓$(RESET) Generated VAULT_MASTER_KEY in $(BACKEND_DIR)/.env"; \
	fi

setup: keys ## Full first-time setup (keys + deps)
	@echo "  Installing frontend dependencies..."
	@cd $(FRONTEND_DIR) && npm install
	@cp -n $(BACKEND_DIR)/.env.example $(BACKEND_DIR)/.env 2>/dev/null || true
	@echo "  $(GREEN)✓$(RESET) Setup complete. Edit $(BACKEND_DIR)/.env then run: make db-up db-migrate"

# ─── Infrastructure ───────────────────────────────────────────────────────────

db-up: ## Start PostgreSQL + Redis via Docker Compose
	docker compose up -d postgres redis
	@echo "  Waiting for PostgreSQL to be ready..."
	@until docker compose exec postgres pg_isready -U authforge -q; do sleep 1; done
	@echo "  $(GREEN)✓$(RESET) PostgreSQL ready"

db-down: ## Stop infrastructure containers
	docker compose down

db-migrate: ## Run database migrations
	cd $(BACKEND_DIR) && sqlx migrate run --source migrations/postgres

db-reset: ## Drop and recreate the database, re-run migrations + seed
	docker compose exec postgres psql -U authforge -d postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'authforge' AND pid <> pg_backend_pid();"
	docker compose exec postgres psql -U authforge -d postgres -c "DROP DATABASE IF EXISTS authforge;"
	docker compose exec postgres psql -U authforge -d postgres -c "CREATE DATABASE authforge OWNER authforge;"
	$(MAKE) db-migrate
	$(MAKE) seed
	@echo "  $(GREEN)✓$(RESET) Database reset complete"

# ─── Development ──────────────────────────────────────────────────────────────

stop: ## Kill all running dev servers (backend + frontend)
	@echo "  Stopping dev servers..."
	@for pid in $$(pgrep -x cargo-watch 2>/dev/null); do kill $$pid 2>/dev/null || true; done
	@for pid in $$(pgrep -x authforge 2>/dev/null); do kill $$pid 2>/dev/null || true; done
	@for pid in $$(pgrep -f "next-server\|next dev" 2>/dev/null); do kill $$pid 2>/dev/null || true; done
	@sleep 2
	@echo "  $(GREEN)✓$(RESET) Dev servers stopped"

kill-ports: ## Kill processes on common dev ports
	@for port in 3000 3001 3002 3003 3004 3005 8080; do \
		pid=$$(ss -tlnp 2>/dev/null | grep ":$$port " | grep -oP 'pid=\K[0-9]+' | head -1); \
		if [ -n "$$pid" ]; then kill $$pid 2>/dev/null || true; echo "  Killed process on port $$port"; fi; \
	done

dev: stop kill-ports ## Start backend + frontend in parallel (kills existing first)
	@echo "  Starting backend and frontend..."
	@$(MAKE) -j2 dev-backend dev-frontend

dev-backend: ## Start Rust backend with auto-reload (requires cargo-watch)
	cd $(BACKEND_DIR) && cargo watch --poll --no-process-group -x "run --bin authforge"

dev-frontend: ## Start Next/refine dev server
	cd $(FRONTEND_DIR) && CHOKIDAR_USEPOLLING=true pnpm dev

# ─── Build ────────────────────────────────────────────────────────────────────

build: build-backend build-frontend ## Build both backend and frontend

build-backend: ## Build Rust release binary
	cd $(BACKEND_DIR) && cargo build --release
	@echo "  $(GREEN)✓$(RESET) Backend binary: $(BACKEND_DIR)/target/release/authforge"

build-frontend: ## Build frontend for production
	cd $(FRONTEND_DIR) && pnpm build
	@echo "  $(GREEN)✓$(RESET) Frontend build: $(FRONTEND_DIR)/.next/"

# ─── Test ─────────────────────────────────────────────────────────────────────

test: test-backend test-frontend ## Run all tests

test-backend: ## Run Rust tests
	cd $(BACKEND_DIR) && cargo test

test-frontend: ## Run frontend type-check
	cd $(FRONTEND_DIR) && pnpm typecheck

test-frontend-e2e: ## Run Playwright E2E tests (requires dev server running)
	cd $(FRONTEND_DIR) && pnpm test:e2e

# ─── Lint ─────────────────────────────────────────────────────────────────────

lint: lint-backend lint-frontend ## Lint both

lint-backend: ## Run clippy
	cd $(BACKEND_DIR) && cargo clippy -- -D warnings

lint-frontend: ## Run ESLint
	cd $(FRONTEND_DIR) && npm run lint

# ─── Seed ─────────────────────────────────────────────────────────────────────

seed: ## Seed database with default roles, permissions, and demo users
	@echo "  Seeding database..."
	cd $(BACKEND_DIR) && cargo run --bin seed
	@echo "  $(GREEN)✓$(RESET) Seed complete"

# ─── Audit ───────────────────────────────────────────────────────────────

audit: audit-backend audit-frontend ## Run security audit on both backend and frontend

audit-backend: ## Run cargo audit on backend dependencies
	cd $(BACKEND_DIR) && cargo audit

audit-frontend: ## Run pnpm audit on frontend dependencies
	cd $(FRONTEND_DIR) && pnpm audit

# ─── Clean ────────────────────────────────────────────────────────────────────

clean: ## Remove build artifacts (keeps keys/ and node_modules)
	cd $(BACKEND_DIR) && cargo clean
	rm -rf $(FRONTEND_DIR)/.next

clean-all: clean ## Remove build artifacts AND node_modules
	rm -rf $(FRONTEND_DIR)/node_modules
