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
    BrokenAddress = 5,
    NotFound = 6,
    Forbidden = 7,
    Slippage = 8,

    // Pool
    ZeroAmount = 101,
    PoolOverflow = 102,
    ZeroChanges = 103,
    NotEnoughAmount = 104,
    InsufficientReceivedAmount = 105,
}
