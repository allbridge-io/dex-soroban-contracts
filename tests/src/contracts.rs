pub mod pool {
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/pool.wasm");
}

pub mod factory {
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/factory.wasm");
}
