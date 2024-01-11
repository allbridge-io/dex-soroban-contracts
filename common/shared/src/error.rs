use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Common
    Unimplemented = 0,
    Initialized = 1,
    Uninitialized = 2,
    Unauthorized = 3,
    InvalidArg = 4,
    InvalidOtherChainId = 6,
    GasUsageNotSet = 7,
    BrokenAddress = 8,
    NotFound = 9,
    Forbidden = 10,
    Slippage = 11,

    // Pool
    ZeroAmount = 101,
    PoolOverflow = 102,
    ZeroChanges = 103,
    InsufficientReceivedAmount = 104,
    NotEnoughAmount = 105,
}
