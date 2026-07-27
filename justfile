# remy - Remote Access Trojan for educational purposes
# Run `just` to see all available commands

# Default recipe - show help
default:
    @just --list

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
build-windows:
    cargo build --release --target x86_64-pc-windows-gnu

# Build frontend (requires pnpm)
build-frontend:
    cd frontend && pnpm install && pnpm build

# Run the C2 server in dev mode
dev:
    cargo run --bin remy -- server

# Run the C2 server with web UI
run:
    cargo run --release --bin remy -- server --ui

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

# Generate a new implant payload
generate os="linux" c2="http://localhost:8080":
    cargo run --release -- generate --os {{os}} --c2 {{c2}}

# Install to ~/.local/bin
install:
    cargo install --path . --root ~/.local

# Show help
help:
    cargo run -- --help
