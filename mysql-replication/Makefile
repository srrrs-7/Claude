.PHONY: setup start stop restart status logs clean test fix reset ssl check help

# Default target
help:
	@echo "MySQL Replication Environment Management"
	@echo ""
	@echo "Usage:"
	@echo "  make setup     - Initialize the environment and set up replication"
	@echo "  make start     - Start all containers"
	@echo "  make stop      - Stop all containers"
	@echo "  make restart   - Restart all containers"
	@echo "  make status    - Show container status"
	@echo "  make logs      - Show logs from all containers"
	@echo "  make test      - Run replication test"
	@echo "  make ssl       - Setup SSL for secure replication"
	@echo "  make fix       - Fix replication issues"
	@echo "  make check     - Check detailed replication status"
	@echo "  make fix-full-db - Fix full database replication (not just test table)"
	@echo "  make deep-fix  - Perform deep fix of replication issues"
	@echo "  make reset     - Complete reset and fix (clears all data)"
	@echo "  make exec-master - Connect to master MySQL shell"
	@echo "  make exec-slave1 - Connect to slave1 MySQL shell"
	@echo "  make exec-slave3 - Connect to slave3 MySQL shell"
	@echo "  make exec-slave3 - Connect to slave3 MySQL shell"
	@echo "  make clean     - Remove all containers and data volumes"

# Setup the environment
setup: ssl start
	@echo "Setting up MySQL replication..."
	@chmod +x ./.scripts/setup-replication.sh ./.scripts/test-replication.sh
	@./.scripts/setup-replication.sh || (echo "\033[0;33mWARNING: Initial setup failed. Attempting to fix...\033[0m" && make fix)

# Check if .env file exists
check-env:
	@echo "Checking for .env file..."
	@if [ ! -f .env ]; then \
		echo "\033[0;33mWARNING: .env file not found, creating from example...\033[0m"; \
		cp .env-example .env; \
		echo "Created .env file with default password. You may want to edit it later."; \
	fi

# Start containers
start:
	@echo "Starting MySQL replication containers..."
	docker compose up -d

# Stop containers
stop:
	@echo "Stopping MySQL replication containers..."
	docker compose stop

# Restart containers
restart:
	@echo "Restarting MySQL replication containers..."
	docker compose restart

# Check containers status
status:
	@echo "MySQL replication containers status:"
	docker compose ps

# Show logs for all containers
logs:
	@echo "Showing logs from all containers..."
	docker compose logs

# Show logs with follow option
logs-follow:
	@echo "Following logs from all containers (Ctrl+C to exit)..."
	docker compose logs -f

# Run replication test
test:
	@echo "Testing MySQL replication..."
	@chmod +x ./.scripts/test-replication.sh
	@./.scripts/test-replication.sh || (echo "\033[0;33mWARNING: Test failed. Attempting to fix replication...\033[0m" && make fix && make test)

# Fix replication issues
fix:
	@echo "Attempting to fix replication issues..."
	@chmod +x ./.scripts/fix-replication.sh
	@./.scripts/fix-replication.sh
	@echo "\033[0;32mReplication issues fixed.\033[0m"

# Check replication status
check:
	@echo "Checking replication status..."
	@chmod +x ./.scripts/check-replication.sh
	@./.scripts/check-replication.sh

# Set up SSL
ssl: check-env
	@echo "Setting up SSL for secure replication..."
	@chmod +x ./.scripts/setup-ssl.sh
	@./.scripts/setup-ssl.sh
	@echo "\033[0;32mSSL setup completed.\033[0m"

# Connect to MySQL master shell
exec-master:
	@echo "Connecting to MySQL master shell..."
	@docker exec -it mysql-master mysql -u root -p

# Connect to MySQL slave1 shell
exec-slave1:
	@echo "Connecting to MySQL slave1 shell..."
	@docker exec -it mysql-slave1 mysql -u root -p

# Connect to MySQL slave2 shell
exec-slave2:
	@echo "Connecting to MySQL slave2 shell..."
	@docker exec -it mysql-slave2 mysql -u root -p

# Connect to MySQL slave3 shell
exec-slave3:
	@echo "Connecting to MySQL slave3 shell..."
	@docker exec -it mysql-slave3 mysql -u root -p

# Clean up
clean:
	@echo "Removing all containers and volumes..."
	docker compose down -v
	@echo "Cleaning data directories..."
	@rm -rf master/data/* slave1/data/* slave2/data/* slave3/data/*
