.DEFAULT_GOAL := all

all: build-factory build-pool

optimize-all: optimize-factory optimize-pool

POOL_WASM_PATH = target/wasm32-unknown-unknown/release/pool.wasm
POOL_WASM_PATH_OP = target/wasm32-unknown-unknown/release/pool.optimized.wasm

FACTORY_WASM_PATH = target/wasm32-unknown-unknown/release/factory.wasm
FACTORY_WASM_PATH_OP = target/wasm32-unknown-unknown/release/factory.optimized.wasm
FACTORY_ADDRESS_PATH = soroban-deploy/factory
FACTORY_ADDRESS=$$(cat $(FACTORY_ADDRESS_PATH))

POOL_WASM_HASH_PATH=soroban-deploy/pool_wasm_hash
POOL_WASM_HASH=$$(cat $(POOL_WASM_HASH_PATH))

ALICE = $$(soroban config identity address alice)
ADMIN_ALIAS = alice
ADMIN = $$(soroban config identity address $(ADMIN_ALIAS))

# YARO:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW
YARO_ADDRESS=CDFVZVTV4K5S66GQXER7YVK6RB23BMPMD3HQUA3TGEZUGDL3NM3R5GDW # Futurenet
# USDY:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW
USDY_ADDRESS=CD7KQQY27G5WXQT2IUYJVHNQH6N2I6GEM5ND2BLZ2GHDAPB2V3KWCW7M # Futurenet

YARO_USDY_POOL=CDF2XUC5BTXFRA6SHRNS2TGHORHXOAS3V7WQJ2OKEBIOQEPCA762OLE4 # Futurenet 

TOKEN_ADDRESS=$(YARO_ADDRESS)
POOL_ADDRESS=$(YARO_USDY_POOL)

NETWORK=futurenet

update-soroban-cli:
	cargo install soroban-cli --features opt

clean-test: clean-target
	make test

clean-target:
	rm -rf target/

lint:
	cargo clippy --all-targets

test: all
	cargo test

build-pool: 
	soroban contract build --package pool

build-factory:
	soroban contract build --package factory

optimize-pool: build-pool
	soroban contract optimize --wasm $(POOL_WASM_PATH)

optimize-factory: build-factory
	soroban contract optimize --wasm $(FACTORY_WASM_PATH)

pool-generate-types:
	soroban contract bindings typescript \
	--network $(NETWORK) \
	--output-dir ./types/pool \
	--wasm $(POOL_WASM_PATH_OP) \
	--contract-id $(POOL_ADDRESS)

#----------------FACTORY----------------------------

install-pool: optimize-pool
	soroban contract install \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) \
		--wasm $(POOL_WASM_PATH_OP)
		> $(POOL_WASM_HASH_PATH) && echo $(POOL_WASM_HASH)

factory-deploy: optimize-factory
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
		--admin $(ADMIN) \
		--wasm-hash $(POOL_WASM_HASH)

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

factory-get-pools:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		pools

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


