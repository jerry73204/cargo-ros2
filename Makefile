.PHONY: build
build:
	cargo build --profile dev-release

.PHONY: test
test:
	cargo nextest run --no-fail-fast --cargo-profile dev-release

.PHONY: clean
clean:
	cargo clean

.PHONY: format
format:
	cargo +nightly fmt

.PHONY: lint
lint:
	cargo clippy --profile dev-release -- -D warnings

.PHONY: check
check:
	cargo check --profile dev-release

.PHONY: doc
doc:
	cargo doc --no-deps --profile dev-release

.PHONY: install
install:
	cargo install --path cargo-ros2
	cargo install --path cargo-ros2-bindgen
