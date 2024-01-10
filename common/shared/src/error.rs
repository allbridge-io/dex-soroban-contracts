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

    // Pool
    ZeroAmount = 103,
    PoolOverflow = 104,
    ZeroChanges = 105,
    ReservesExhausted = 106,
    InsufficientReceivedAmount = 107,
    BalanceRatioExceeded = 108,
    Forbidden = 109,
}
