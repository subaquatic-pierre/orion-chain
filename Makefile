
.PHONY: all

export RUST_LOG=debug

build:
	cargo run build --release

run: 
	cargo run

test:
	cargo test -- --nocapture