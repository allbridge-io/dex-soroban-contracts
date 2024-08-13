.DEFAULT_GOAL := all

all: build-two-pool build-three-pool build-factory

optimize-all: optimize-factory optimize-two-pool optimize-three-pool

TWO_POOL_WASM_PATH = target/wasm32-unknown-unknown/release/pool.wasm
TWO_POOL_WASM_PATH_OP = target/wasm32-unknown-unknown/release/pool.optimized.wasm

THREE_POOL_WASM_PATH = target/wasm32-unknown-unknown/release/three_pool.wasm
THREE_POOL_WASM_PATH_OP = target/wasm32-unknown-unknown/release/three_pool.optimized.wasm

FACTORY_WASM_PATH = target/wasm32-unknown-unknown/release/factory.wasm
FACTORY_WASM_PATH_OP = target/wasm32-unknown-unknown/release/factory.optimized.wasm
FACTORY_ADDRESS=CCB7MOTLIZH32HOZP5NIKYUH6UHDKZAW3YLFAGXXTMHU75Z2A2AVWNHV

TWO_POOL_WASM_HASH=b0adafcf2b3f0f66b9f56f0b441c0d6cd19e9cd9550e294a6e7fed868f17f34d
THREE_POOL_WASM_HASH=ca57c911473636d76059a8ef826a1f2305d72a3c6df609aab9042486d1d38467

ALICE = $$(soroban keys address alice)
ADMIN_ALIAS = alice
ADMIN = $$(soroban keys address $(ADMIN_ALIAS))
DEPLOYER=$(ADMIN)

# YARO:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW
YARO_ADDRESS=CACOK7HB7D7SRPMH3LYYOW77T6D4D2F7TR7UEVKY2TVSUDSRDM6DZVLK# Testnet
# USDY:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW
USDY_ADDRESS=CAOPX7DVI3PFLHE7637YSFU6TLG6Z27Z5O3M547ANAYXQOAYCYYV6NO6# Testnet
# BOGD:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW
BOGD_ADDRESS=CDBDW5BMDBFQGKI4UWUFZQEO7OKFTGNLU5BV2I3DKPJ33OWMKLERRMS6# Testnet

YUSD_YARO_BOGD_POOL=CCIFAKJVJGROBM5V57HOPWJC2O5RTYMLZBMPBJLOLMYRPGVPD3OUEIKP # Testnet
YUSD_YARO_POOL=CDS3O7QRJ6646LKP3LTU7NU4BVODCPZNHVQTYPMKTJ5DZXJBJKZJMRGE # Testnet

TOKEN_ADDRESS=$(BOGD_ADDRESS)
POOL_ADDRESS=$(YUSD_YARO_BOGD_POOL)

NETWORK=testnet

prepare: update-soroban-cli
	rustup target add wasm32-unknown-unknown

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

build-two-pool:
	soroban contract build --package pool

build-three-pool:
	soroban contract build --package three_pool

build-factory:
	soroban contract build --package factory

optimize-two-pool: build-two-pool
	soroban contract optimize --wasm $(TWO_POOL_WASM_PATH)

optimize-three-pool: build-three-pool
	soroban contract optimize --wasm $(THREE_POOL_WASM_PATH)

optimize-factory: build-factory
	soroban contract optimize --wasm $(FACTORY_WASM_PATH)

pool-generate-types:
	soroban contract bindings typescript \
	--network $(NETWORK) \
	--output-dir ./types/pool \
	--wasm $(POOL_WASM_PATH_OP) \
	--contract-id $(POOL_ADDRESS)

#----------------FACTORY----------------------------

install-two-pool: optimize-two-pool
	soroban contract install \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) \
		--wasm $(TWO_POOL_WASM_PATH_OP)

install-three-pool: optimize-three-pool
	soroban contract install \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) \
		--wasm $(THREE_POOL_WASM_PATH_OP)

factory-deploy: optimize-factory
	soroban contract deploy \
		--wasm $(FACTORY_WASM_PATH_OP) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK)

factory-initialize:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		initialize \
		--admin $(ADMIN) \
		--two-pool-wasm-hash $(TWO_POOL_WASM_HASH) \
		--three-pool-wasm-hash $(THREE_POOL_WASM_HASH)

factory-create-pool:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		create_pool \
		--deployer $(DEPLOYER) \
		--pool-admin $(ADMIN) \
		--a 20 \
		--tokens '["$(USDY_ADDRESS)", "$(YARO_ADDRESS)"]' \
		--fee_share_bp 15 \
		--admin-fee-share-bp 2000

factory-update-two-pool-wasm-hash:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		update_two_pool_wasm_hash \
		--new_wasm_hash $(TWO_POOL_WASM_HASH)

factory-update-three-pool-wasm-hash:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		update_three_pool_wasm_hash \
		--new_wasm_hash $(THREE_POOL_WASM_HASH)

factory-get-pool:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
		-- \
		pool \
		--tokens '["$(YARO_ADDRESS)", "$(USDY_ADDRESS)", "$(BOGD_ADDRESS)"]'

factory-get-pools:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
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
		--amounts '["1000000000000", "1000000000000", "1000000000000"]' \
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
		--is-view \
		-- \
		get_pool

pool-pending-reward:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
		-- \
		pending_reward \
		--user $(ADMIN)

pool-get-d:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
		-- \
		get_d

pool-get-withdraw-amount:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
		-- \
		get_withdraw_amount \
		--lp_amount 100000

pool-get-deposit-amount:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
		-- \
		get_deposit_amount \
		--amounts '["100000", "100000", "100000"]'

pool-swap:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
		-- \
		swap \
		--sender $(ALICE) \
		--token_from 0 \
		--token_to 1 \
		--amount_in 10000000 \
		--recipient $(ALICE) \
		--receive_amount_min 0


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
		--source $(ADMIN_ALIAS) \
		--is-view \
		-- \
		balance \
		--id $(POOL_ADDRESS)


token-get-name:
	soroban contract invoke \
		--id $(TOKEN_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
		-- \
		name

wrap-token:
	soroban contract asset deploy \
		--network $(NETWORK) 	\
		--source  $(ADMIN_ALIAS) \
		--asset BOGD:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW

native-token-address:
	soroban contract asset id \
		--network $(NETWORK) \
		--source $(ADMIN_ALIAS) \
		--asset native


