.DEFAULT_GOAL := all

all: build-pool

POOL_WASM_PATH = target/wasm32-unknown-unknown/release/pool.wasm
POOL_WASM_PATH_OP = target/wasm32-unknown-unknown/release/pool.optimized.wasm

clean-test: clean-target
	make test

clean-target:
	rm -rf target/

test: all
	cargo test

build-pool: 
	cargo build --target wasm32-unknown-unknown --release --package pool

optimize-pool:
	soroban contract optimize --wasm $(POOL_WASM_PATH)
