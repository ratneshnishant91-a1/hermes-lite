.PHONY: build test lint run docker-build docker-run clean

# Build Rust binary
build:
	cargo build --release

# Run tests
test:
	cargo test -- --nocapture

# Lint with clippy
lint:
	cargo clippy --all-targets -- -D warnings

# Run interactive CLI
run:
	cargo run --release -- chat

# Build Docker image (distroless)
docker-build:
	docker build -t hermes-lite:latest .

# Run Docker container
docker-run:
	docker run --rm -it \
		-e OPENAI_API_KEY=$(OPENAI_API_KEY) \
		-p 8000:8000 \
		hermes-lite:latest

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt --check

# Full CI check
ci: fmt-check lint test build
