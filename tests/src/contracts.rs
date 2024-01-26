pub mod pool {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/contract-release/pool.wasm"
    );
}

pub mod factory {
    #![allow(clippy::too_many_arguments)]
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/contract-release/factory.wasm"
    );
}
