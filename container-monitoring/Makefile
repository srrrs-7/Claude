.PHONY: build run test clean docker-build docker-run docker-stop docker-clean docker-compose-up docker-compose-down docker-compose-logs lint fmt help cleanup-mod-rs

# デフォルトターゲット
all: help

# プロジェクト名とバージョン
PROJECT_NAME := container-monitoring
VERSION := $(shell grep '^version' Cargo.toml | cut -d '"' -f2)

# ビルド関連
CARGO := cargo
DOCKER := docker
DOCKER_COMPOSE := docker compose
RUST_LOG ?= info

# ローカル開発用コマンド
build:
	@echo "Building $(PROJECT_NAME) v$(VERSION)..."
	$(CARGO) build --release

run:
	@echo "Running $(PROJECT_NAME) v$(VERSION)..."
	RUST_LOG=$(RUST_LOG) $(CARGO) run --release -- --config config/config.toml

dev:
	@echo "Running in development mode with auto-reload..."
	RUST_LOG=debug cargo watch -x 'run -- --config config/config.toml'

test:
	@echo "Running tests..."
	$(CARGO) test

fmt:
	@echo "Formatting code..."
	$(CARGO) fmt

lint:
	@echo "Linting code..."
	$(CARGO) clippy -- -D warnings

clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf target/

# Docker関連コマンド
docker-build:
	@echo "Building Docker image..."
	$(DOCKER) build -t $(PROJECT_NAME):$(VERSION) .

docker-run: docker-build
	@echo "Running Docker container..."
	$(DOCKER) run -d --name $(PROJECT_NAME) \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-p 8080:8080 \
		-e RUST_LOG=$(RUST_LOG) \
		$(PROJECT_NAME):$(VERSION)

docker-stop:
	@echo "Stopping Docker container..."
	$(DOCKER) stop $(PROJECT_NAME) || true
	$(DOCKER) rm $(PROJECT_NAME) || true

docker-clean: docker-stop
	@echo "Removing Docker image..."
	$(DOCKER) rmi $(PROJECT_NAME):$(VERSION) || true

# Docker Compose関連コマンド
docker-compose-up:
	@echo "Starting all services with Docker Compose..."
	$(DOCKER_COMPOSE) up -d

docker-compose-down:
	@echo "Stopping all services..."
	$(DOCKER_COMPOSE) down

docker-compose-build:
	@echo "Building all services..."
	$(DOCKER_COMPOSE) build

docker-compose-logs:
	@echo "Showing logs..."
	$(DOCKER_COMPOSE) logs -f

# 開発環境セットアップ
setup-dev:
	@echo "Setting up development environment..."
	rustup update
	cargo install cargo-watch cargo-edit cargo-audit cargo-outdated

# ユーティリティコマンド
check-deps:
	@echo "Checking for outdated dependencies..."
	$(CARGO) outdated

audit:
	@echo "Auditing dependencies for security vulnerabilities..."
	$(CARGO) audit

update-deps:
	@echo "Updating dependencies..."
	$(CARGO) update

# ダッシュボードへのクイックアクセス（URLを表示）
open-dashboards:
	@echo "Dashboard URLs:"
	@echo "  - Grafana:    http://localhost:3000 (admin/admin)"
	@echo "  - Prometheus: http://localhost:9090"
	@echo "  - Metrics:    http://localhost:8080/metrics"
	@echo "  - Health:     http://localhost:8080/health"

# Prometheusのクエリ例を表示
prometheus-queries:
	@echo "Useful Prometheus Queries:"
	@echo "  - CPU usage:      container_cpu_usage_percent"
	@echo "  - Memory usage:   container_memory_usage_bytes"
	@echo "  - Network RX:     rate(container_network_receive_bytes_total[1m])"
	@echo "  - Network TX:     rate(container_network_transmit_bytes_total[1m])"
	@echo "  - Disk reads:     rate(container_fs_reads_bytes_total[1m])"
	@echo "  - Disk writes:    rate(container_fs_writes_bytes_total[1m])"
	@echo "  - Container count: container_count"

# mod.rsの代わりに.rsファイルを使う構造に変更
cleanup-mod-rs:
	@echo "古いmod.rsファイルとディレクトリを削除しています..."
	bash rm_old_files.sh

# ヘルプ表示
help:
	@echo "$(PROJECT_NAME) v$(VERSION) Makefile"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Development Targets:"
	@echo "  build              Build the project"
	@echo "  run                Run the project locally"
	@echo "  dev                Run with auto-reload (requires cargo-watch)"
	@echo "  test               Run tests"
	@echo "  fmt                Format code with rustfmt"
	@echo "  lint               Lint code with clippy"
	@echo "  clean              Clean build artifacts"
	@echo "  setup-dev          Install development tools"
	@echo ""
	@echo "Docker Targets:"
	@echo "  docker-build       Build Docker image"
	@echo "  docker-run         Run Docker container"
	@echo "  docker-stop        Stop and remove Docker container"
	@echo "  docker-clean       Remove Docker image"
	@echo ""
	@echo "Docker Compose Targets:"
	@echo "  docker-compose-up        Start all services"
	@echo "  docker-compose-down      Stop all services"
	@echo "  docker-compose-build     Build all services"
	@echo "  docker-compose-logs      Show logs from all services"
	@echo ""
	@echo "Utility Targets:"
	@echo "  check-deps         Check for outdated dependencies"
	@echo "  audit              Audit dependencies for security vulnerabilities"
	@echo "  update-deps        Update dependencies"
	@echo "  open-dashboards    Show URLs for monitoring dashboards"
	@echo "  prometheus-queries Show useful Prometheus queries"
	@echo "  cleanup-mod-rs     Remove old mod.rs files and convert to flat structure"