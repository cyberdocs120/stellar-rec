use soroban_sdk::contracterror;

#[derive(Copy, Clone, Debug, PartialEq)]
#[contracterror]
pub enum RecTokenError {
    Unauthorized = 1,
    RecAlreadyRetired = 2,
    InsufficientBalance = 3,
    TokenNotFound = 4,
    DuplicateMint = 5,
    ContractPaused = 6,
}
