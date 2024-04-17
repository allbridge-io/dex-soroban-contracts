.DEFAULT_GOAL := all

all: build-factory build-pool

optimize-all: optimize-factory optimize-pool

POOL_WASM_PATH = target/wasm32-unknown-unknown/release/pool.wasm
POOL_WASM_PATH_OP = target/wasm32-unknown-unknown/release/pool.optimized.wasm

FACTORY_WASM_PATH = target/wasm32-unknown-unknown/release/factory.wasm
FACTORY_WASM_PATH_OP = target/wasm32-unknown-unknown/release/factory.optimized.wasm
# FACTORY_ADDRESS=CACBPPNXJZZECUERFKHDPHWCLA6KEWE5N45ER3E3SPTXZ34JVVTTNV2N # Testnet
FACTORY_ADDRESS=CBBGMOA67G4Y3KE7TTU5TCUZFXEQWCFZVFZHN3W4EX3ESYBMFK6HSZHS

# POOL_WASM_HASH=5126675e4652dadd2724df5ea02f191a3cfbd0447ba47783c3a34bf302493a0d

# POOL_WASM_HASH=f8287b3877f6a4a951d889a57557b8e2ce890e3917f63a1e938bf33c9069e25a
POOL_WASM_HASH=52b565b63df831f0bcfda198f1be95981b85ca2cf9e4a4eb7c2ed21047a340d0

ALICE = $$(soroban config identity address alice)
ADMIN_ALIAS = alice
ADMIN = $$(soroban config identity address $(ADMIN_ALIAS))
DEPLOYER=$(ADMIN)

# YARO:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW
YARO_ADDRESS=CACOK7HB7D7SRPMH3LYYOW77T6D4D2F7TR7UEVKY2TVSUDSRDM6DZVLK # Testnet
# USDY:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW
USDY_ADDRESS=CAOPX7DVI3PFLHE7637YSFU6TLG6Z27Z5O3M547ANAYXQOAYCYYV6NO6 # Testnet
# BOGD:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW
BOGD_ADDRESS=CDBDW5BMDBFQGKI4UWUFZQEO7OKFTGNLU5BV2I3DKPJ33OWMKLERRMS6 # Testnet

JUSD_ADDRESS=CCJ7EPBMUIWARLV3KMGZGBW2H7J4O2UOMJY6F74GHTHSKGPHULL62UPJ
PUSD_ADDRESS=CD4ZZAWCIPZ4OWLJCMAMCDHD67GL37P6KSFDRKJEXS43UCGT6Y4BIER6

YARO_BOGD_POOL=CDPIBT5DBMOXBE5AT6IVFWO36CMDOBNR6LWRECD3MNDCDHBLJLB7PEHN # Testnet 
USDY_BOGD_POOL=CBZGPFLKGVMIDQD5N3N5HWZCHPIBIFACKQXWIPCOCUYGBDLW37FTSIWE # Testnet 
YARO_JUSD_POOL=CB3PASPY75CM5IJQYWLFFQRLJSANG7IFYZUGKMQAID4A3ASSEW7HWAGY

TOKEN_ADDRESS=$(BOGD_ADDRESS)
POOL_ADDRESS=$(YARO_JUSD_POOL)

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
		--wasm-hash $(POOL_WASM_HASH)

factory-create-pair:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		create_pair \
		--deployer $(DEPLOYER) \
		--pool-admin $(ADMIN) \
		--a 20 \
		--token-a $(JUSD_ADDRESS) \
		--token-b $(YARO_ADDRESS) \
		--fee_share_bp 15 \
		--admin-fee-share-bp 2000

factory-update-wasm-hash:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		update_wasm_hash \
		--new_wasm_hash $(POOL_WASM_HASH) 

factory-get-pool:
	soroban contract invoke \
		--id $(FACTORY_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		--is-view \
		-- \
		pool \
		--token-a $(YARO_ADDRESS) \
		--token-b $(USDY_ADDRESS)

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
		--is-view \
		-- \
		get_pool

pool-pending-reward:
	soroban contract invoke \
		--id $(POOL_ADDRESS) \
		--source $(ADMIN_ALIAS) \
		--network $(NETWORK) 	\
		-- \
		--is-view \
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
		--amounts '["100000", "100000"]'

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
	soroban lab token wrap \
		--network $(NETWORK) 	\
		--asset USDY:GAYODJWF27E5OQO2C6LA6Z6QXQ2EYUONMXFNL2MNMGRJP6RED2CPQKTW

native-token-address:
	soroban lab token id \
 		--network $(NETWORK) \
 		--asset native


