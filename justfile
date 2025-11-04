# Show available commands (default)
default:
	@just --list

# Build with dev-release profile
build:
	cargo build --profile dev-release

# Run tests with nextest
test:
	cargo nextest run --no-fail-fast --cargo-profile dev-release

# Clean build artifacts
clean:
	cargo clean

# Format code with nightly
format:
	cargo +nightly fmt

# Run clippy with warnings as errors
lint:
	cargo clippy --profile dev-release -- -D warnings

# Check code without building
check:
	cargo check --profile dev-release

# Generate documentation
doc:
	cargo doc --no-deps --profile dev-release

# Install cargo-ros2 (cargo) and colcon-ros-cargo (pip)
install:
	cargo install --path cargo-ros2
	cargo install --path cargo-ros2-bindgen
	pip install --break-system-packages --editable colcon-ros-cargo

# Run all quality checks (format + lint)
quality: format lint

# Full CI workflow (format + lint + test)
ci: format lint test
