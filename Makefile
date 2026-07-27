# remy - Remote Access Trojan for educational purposes
# Run `make help` to see all available commands

.PHONY: all build build-server build-implant build-windows build-frontend \
        dev run test fmt lint clean install help

# Default target
all: build

# Build everything (server + implant) in release mode
build:
	cargo build --release

# Build only the C2 server
build-server:
	cargo build --release --bin remy

# Build only the implant
build-implant:
	cargo build --release --bin remy-implant

# Build for Windows (cross-compile)
# Requires: rustup target add x86_64-pc-windows-gnu
build-windows:
	cargo build --release --target x86_64-pc-windows-gnu

# Build frontend (requires pnpm)
build-frontend:
	cd frontend && pnpm install && pnpm build

# Run the C2 server in dev mode
dev:
	cargo run -- server

# Run the C2 server with web UI
run:
	cargo run --release -- server --ui

# Run tests
test:
	cargo test

# Format code
fmt:
	cargo fmt

# Lint code
lint:
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	rm -rf frontend/dist frontend/node_modules

# Install to ~/.local/bin
install:
	cargo install --path . --root ~/.local

# Show available commands
help:
	@echo "remy - Remote Access Trojan (Educational)"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@echo "  build          Build everything (release mode)"
	@echo "  build-server   Build only C2 server"
	@echo "  build-implant  Build only implant"
	@echo "  build-windows  Cross-compile for Windows"
	@echo "  build-frontend Build web UI"
	@echo "  dev            Run server in dev mode"
	@echo "  run            Run server with web UI"
	@echo "  test           Run tests"
	@echo "  fmt            Format code"
	@echo "  lint           Lint code"
	@echo "  clean          Clean build artifacts"
	@echo "  install        Install to ~/.local/bin"
