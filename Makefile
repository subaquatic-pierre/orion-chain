
.PHONY: all

export RUST_LOG=debug

build:
	cargo run build --release

run: 
	cargo run

dev:
	cargo watch -x "run --bin orion-chain" 

test:
	cargo test -- --nocapture