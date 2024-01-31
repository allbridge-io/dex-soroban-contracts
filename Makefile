.DEFAULT_GOAL := all

all: build-factory

optimize-all: optimize-factory optimize-pool

POOL_WASM_PATH = target/wasm32-unknown-unknown/release/pool.wasm
POOL_WASM_PATH_OP = target/wasm32-unknown-unknown/release/pool.optimized.wasm

FACTORY_WASM_PATH = target/wasm32-unknown-unknown/release/factory.wasm
FACTORY_WASM_PATH_OP = target/wasm32-unknown-unknown/release/factory.optimized.wasm
FACTORY_ADDRESS_PATH = soroban-deploy/factory
FACTORY_ADDRESS=$$(cat $(FACTORY_ADDRESS_PATH))

ALICE = $$(soroban config identity address alice)
ADMIN_ALIAS = alice
ADMIN = $$(soroban config identity address $(ADMIN_ALIAS))

YARO_ADDRESS=CACOK7HB7D7SRPMH3LYYOW77T6D4D2F7TR7UEVKY2TVSUDSRDM6DZVLK #Testnet
USDY_ADDRESS=CAOPX7DVI3PFLHE7637YSFU6TLG6Z27Z5O3M547ANAYXQOAYCYYV6NO6 #Testnet

YARO_USDY_POOL=CDUUO74BWN25FLOV4JF6EGQZPRPGLWSY7K7TVJZDJJFV4KCMSRTVYD52 # Testnet

TOKEN_ADDRESS=$(USDY_ADDRESS)
POOL_ADDRESS=$(YARO_USDY_POOL)

NETWORK=testnet

clean-test: clean-target
	make test

clean-target:
	rm -rf target/

lint:
	cargo clippy --all-targets

test: all
	cargo test

build-pool: 
	cargo build --target wasm32-unknown-unknown --release --package pool

build-factory: build-pool
	cargo build --target wasm32-unknown-unknown --release --package factory

optimize-pool:
	soroban contract optimize --wasm $(POOL_WASM_PATH)

optimize-factory:
	soroban contract optimize --wasm $(FACTORY_WASM_PATH)

fuzz:
	cargo run --bin fuzz
#	cargo run --bin fuzz -- --runs 10  --run-len 50 --threads 8

pool-generate-types:
	soroban contract bindings typescript \
	--network $(NETWORK) \
	--output-dir ./types/pool \
	--wasm $(POOL_WASM_PATH_OP) \
	--contract-id $(POOL_ADDRESS)

#----------------FACTORY----------------------------

factory-deploy:
	soroban contract deploy \
		--wasm $(FACTORY_WASM_PATH_OP) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) \
		> $(FACTORY_ADDRESS_PATH) && echo $(FACTORY_ADDRESS)

factory-initialize:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		initialize \
		--admin $(ADMIN)

factory-create-pair:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		create_pair \
		--deployer $(ADMIN) \
		--pool-admin $(ADMIN) \
		--a 20 \
		--token-a $(YARO_ADDRESS) \
		--token-b $(USDY_ADDRESS) \
		--fee_share_bp 10 \
		--admin-fee-share-bp 10

factory-get-pool:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		pool \
		--token-a $(YARO_ADDRESS) \
		--token-b $(USDY_ADDRESS)

#----------------POOL----------------------------

pool-deposit:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		deposit \
		--sender $(ADMIN) \
		--amounts '["1000000000000", "1000000000000"]' \
		--min-lp-amount 1000

pool-withdraw:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		withdraw \
		--sender $(ADMIN) \
		--lp-amount 100000

pool-claim-rewards:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		claim_rewards \
		--sender $(ADMIN)
		
pool-get-pool-info:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		get_pool

pool-pending-reward:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		pending_reward \
		--user $(ADMIN)

pool-get-d:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		get_d

#----------TOKEN--------------------------

token-transfer:
	soroban contract invoke \
		--id $(TOKEN_ADDRESS) \
		--source SBTECKZAIBLA6ZGPCG5IKON2IG4SJ37AVZEIY5OHCCKJ7KYCAJQKF5EB \
		--network $(NETWORK) 	\
		-- \
		transfer \
		--from GA2LLFIX5V3JT6IW67HH2JESPYALDGCV2AGCSEQOOEMKMF5K3WL2K7OS \
		--to $(ADMIN) \
		--amount 10000000000000

token-native-transfer:
	soroban contract invoke \
		--id $(NATIVE_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		transfer \
		--from $(ADMIN) \
		--to $(BRIDGE_ADDRESS) \
		--amount 1000000000

token-get-balance:
	soroban contract invoke \
		--id $(TOKEN_ADDRESS) \
		--network $(NETWORK) 	\
		-- \
		balance \
		--id $(POOL_ADDRESS)


token-get-name:
	soroban contract invoke \
		--id $(TOKEN_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		name

wrap-token:
	soroban lab token wrap \
		--network $(NETWORK) 	\
		--asset USDY:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW

native-token-address:
	soroban lab token id \
 		--network $(NETWORK) \
 		--asset native


