pub mod two_pool {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release-with-logs/two_pool.wasm");
}

pub mod three_pool {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release-with-logs/three_pool.wasm"
    );
}

pub mod factory {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release-with-logs/factory.wasm");
}

// pub mod two_pool {
//     soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/two_pool.wasm");
// }
//
// pub mod three_pool {
//     soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/three_pool.wasm");
// }
//
// pub mod factory {
//     soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/factory.wasm");
// }
