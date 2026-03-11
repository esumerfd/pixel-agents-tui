.PHONY: build release check clean run setup ci ci-release help

build:
	cargo build

release:
	cargo build --release

check:
	cargo check && cargo clippy -- -A clippy::all -D clippy::correctness

clean:
	cargo clean

run:
	cargo run

setup:
	rustup target add aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu

ci:
	gh workflow run main.yml

ci-release:
	gh workflow run release.yml

help:
	@echo "Targets:"
	@echo "  build    Debug build (host arch)"
	@echo "  release  Release build (host arch)"
	@echo "  check    cargo check + clippy"
	@echo "  clean    Remove build artefacts"
	@echo "  run      Run pixel-agents-tui (debug)"
	@echo "  setup    Add cross-compile rustup targets"
	@echo ""
	@echo "CI targets:"
	@echo "  ci           Trigger Build workflow on GitHub"
	@echo "  ci-release   Trigger Release workflow on GitHub"
