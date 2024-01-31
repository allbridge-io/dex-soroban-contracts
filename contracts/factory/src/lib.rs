#![no_std]

mod contract;
mod events;
mod methods;
mod storage;

mod pool {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/pool.wasm");
}

pub use contract::FactoryContract;
