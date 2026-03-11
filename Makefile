.PHONY: build release test check clean run setup help

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

check:
	cargo check && cargo clippy -- -D warnings

clean:
	cargo clean

run:
	cargo run

fmt:
	cargo fmt

setup:
	rustup target add aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu

help:
	@echo "Targets:"
	@echo "  build    Debug build (host arch)"
	@echo "  release  Release build (host arch)"
	@echo "  test     Run all tests"
	@echo "  check    cargo check + clippy"
	@echo "  clean    Remove build artefacts"
	@echo "  run      Run pixel-agents-tui (debug)"
	@echo "  fmt      Format code"
	@echo "  setup    Add cross-compile rustup targets"
