.PHONY: test test-unit test-integration test-local test-real setup-test-server teardown-test-server build clean

# Default target
all: build

# Build the project
build:
	cargo build

# Build release version
build-release:
	cargo build --release

# Run all tests
test: test-unit test-integration

# Run unit tests only
test-unit:
	cargo test --lib

# Run integration tests with local server
test-local: setup-test-server
	@echo "Waiting for test server to start..."
	@sleep 2
	cargo test test_ping_local_test_server -- --nocapture
	$(MAKE) teardown-test-server

# Run integration tests against real servers
test-real:
	@echo "Running tests against real servers (requires internet)..."
	cargo test integration_tests -- --nocapture

# Run all integration tests (local + real)
test-integration: test-local test-real

# Start test server in background
setup-test-server:
	@echo "Starting test server..."
	@cargo build --bin test_server
	@cargo run --bin test_server &
	@echo $$! > test_server.pid
	@echo "Test server started with PID $$(cat test_server.pid)"

# Stop test server
teardown-test-server:
	@if [ -f test_server.pid ]; then \
		echo "Stopping test server with PID $$(cat test_server.pid)"; \
		kill $$(cat test_server.pid) 2>/dev/null || true; \
		rm -f test_server.pid; \
	else \
		echo "No test server PID file found"; \
	fi

# Docker-based test server (alternative)
setup-test-server-docker:
	@echo "Starting test server with Docker..."
	docker run -d --name latencee-test-server -p 8080:8080 nginx:alpine
	@echo "Docker test server started"

teardown-test-server-docker:
	@echo "Stopping Docker test server..."
	docker stop latencee-test-server 2>/dev/null || true
	docker rm latencee-test-server 2>/dev/null || true

# Test with Docker server
test-local-docker: setup-test-server-docker
	@echo "Waiting for Docker test server to start..."
	@sleep 3
	cargo test test_ping_local_test_server -- --nocapture
	$(MAKE) teardown-test-server-docker

# Clean build artifacts
clean:
	cargo clean
	rm -f test_server.pid

# Run the main application
run:
	cargo run

# Run in release mode
run-release:
	cargo run --release

# Check code without building
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run clippy linter
clippy:
	cargo clippy

# Run all quality checks
quality: fmt clippy check

# Help target
help:
	@echo "Available targets:"
	@echo "  build                 - Build the project"
	@echo "  build-release         - Build release version"
	@echo "  test                  - Run all tests"
	@echo "  test-unit             - Run unit tests only"
	@echo "  test-local            - Run tests with local server"
	@echo "  test-real             - Run tests against real servers"
	@echo "  test-integration      - Run all integration tests"
	@echo "  setup-test-server     - Start local test server"
	@echo "  teardown-test-server  - Stop local test server"
	@echo "  test-local-docker     - Run tests with Docker server"
	@echo "  run                   - Run the application"
	@echo "  run-release           - Run release version"
	@echo "  clean                 - Clean build artifacts"
	@echo "  quality               - Run fmt, clippy, and check"
	@echo "  help                  - Show this help"