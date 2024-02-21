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
    CastFailed = 9,
    TokenInsufficientBalance = 10,

    // Pool
    ZeroAmount = 100,
    PoolOverflow = 101,
    ZeroChanges = 102,
    NotEnoughAmount = 103,
    InsufficientReceivedAmount = 104,
    Slippage = 105,
    InvalidFirstDeposit = 106,

    // Factory
    PairExist = 200,
    IdenticalAddresses = 201,
}
