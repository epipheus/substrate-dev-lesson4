run-tmp:
	cargo run -- --dev --tmp

run:
	cargo run -- --dev

toolchain:
	./scripts/init.sh

build:
	CARGO_INCREMENTAL=0 cargo build

check:
	CARGO_INCREMENTAL=0 SKIP_WASM_BUILD= cargo check --all --tests

test:
	CARGO_INCREMENTAL=0 SKIP_WASM_BUILD= cargo test --all

purge:
	cargo run -- purge-chain --dev -y

restart: purge run

init: toolchain build-full
