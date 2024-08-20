pub mod pool {
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/two_pool.wasm");
}

pub mod three_pool {
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/three_pool.wasm");
}

pub mod factory {
    #![allow(clippy::too_many_arguments)]
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/factory.wasm");
}
