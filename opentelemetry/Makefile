.PHONY: build run-server run-client docker-up docker-down clean test

# Binary names
SERVER_BIN=bin/server
CLIENT_BIN=bin/client

# Go binary
GO=$(shell which go)

# Output directories
BIN_DIR=bin

# Default target
all: build

# Create output directories
$(BIN_DIR):
	mkdir -p $(BIN_DIR)

# Build the server and client binaries
build: $(BIN_DIR)
	$(GO) build -o $(SERVER_BIN) ./cmd/server
	$(GO) build -o $(CLIENT_BIN) ./cmd/client

# Run the server
run-server:
	$(GO) run ./cmd/server

# Run the client
run-client:
	$(GO) run ./cmd/client

# Docker targets
setup-deps:
	go mod tidy

docker-build: setup-deps
	docker-compose build

docker-up: docker-build
	docker-compose up -d

docker-logs:
	docker-compose logs -f

docker-down:
	docker-compose down

# Start everything in docker (for convenience)
docker-start: docker-up

# Clean build artifacts
clean:
	rm -rf $(BIN_DIR)

# Run tests
test:
	$(GO) test -v ./...

# Tidy go modules
tidy:
	$(GO) mod tidy

# Download dependencies
deps:
	$(GO) mod download

# Start everything (for convenience)
start: docker-up run-server

# Stop everything (for convenience)
stop: docker-down

# Help command
help:
	@echo "Available commands:"
	@echo "  make build        - Build the server and client binaries"
	@echo "  make run-server   - Run the server"
	@echo "  make run-client   - Run the client"
	@echo "  make docker-build - Build docker images"
	@echo "  make docker-up    - Start all services in docker"
	@echo "  make docker-logs  - Show logs for all docker services"
	@echo "  make docker-down  - Stop all docker services" 
	@echo "  make docker-start - Build and start all docker services"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make test         - Run tests"
	@echo "  make tidy         - Tidy go modules"
	@echo "  make deps         - Download dependencies"
	@echo "  make start        - Start everything (docker + server)"
	@echo "  make stop         - Stop everything"
