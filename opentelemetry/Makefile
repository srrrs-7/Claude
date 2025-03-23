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

# Start docker compose services
docker-up:
	docker-compose up -d

# Stop docker compose services
docker-down:
	docker-compose down

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
	@echo "  make docker-up    - Start docker compose services"
	@echo "  make docker-down  - Stop docker compose services" 
	@echo "  make clean        - Clean build artifacts"
	@echo "  make test         - Run tests"
	@echo "  make tidy         - Tidy go modules"
	@echo "  make deps         - Download dependencies"
	@echo "  make start        - Start everything (docker + server)"
	@echo "  make stop         - Stop everything"
