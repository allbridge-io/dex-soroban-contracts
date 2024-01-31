```sh
cargo install soroban-cli --features opt
```

Autocomplete

```sh
echo "source <(soroban completion --shell bash)" >> ~/.bashrc
```

### Configuring the CLI for Testnet

```sh
soroban config network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

#### Configure an Identity

```sh
soroban config identity generate --global alice
```

Airdrop:

```sh
curl "https://friendbot.stellar.org/?addr=$(soroban config identity address alice)"
```
