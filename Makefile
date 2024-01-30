.DEFAULT_GOAL := all

all: build-factory

POOL_WASM_PATH = target/wasm32-unknown-unknown/contract-release/pool.wasm
POOL_WASM_PATH_OP = target/wasm32-unknown-unknown/contract-release/pool.optimized.wasm

FACTORY_WASM_PATH = target/wasm32-unknown-unknown/contract-release/factory.wasm
FACTORY_WASM_PATH_OP = target/wasm32-unknown-unknown/contract-release/factory.optimized.wasm

clean-test: clean-target
	make test

clean-target:
	rm -rf target/

lint:
	cargo clippy --all-targets

test: all
	cargo test

build-pool: 
	cargo build --target wasm32-unknown-unknown --profile contract-release --package pool

build-factory: build-pool
	cargo build --target wasm32-unknown-unknown --profile contract-release --package factory

optimize-pool:
	soroban contract optimize --wasm $(POOL_WASM_PATH)

optimize-factory:
	soroban contract optimize --wasm $(FACTORY_WASM_PATH)

fuzz:
	cargo run --bin fuzz
