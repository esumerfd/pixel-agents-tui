.PHONY: build run clean check fmt clippy test

build:
	cargo build

run: build
	cargo run

clean:
	cargo clean

check:
	cargo check

fmt:
	cargo fmt

clippy:
	cargo clippy

test:
	cargo test
