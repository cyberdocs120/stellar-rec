use soroban_sdk::contracterror;

#[derive(Copy, Clone, Debug, PartialEq)]
#[contracterror]
pub enum RetirementError {
    Unauthorized = 1,
    AlreadyRetired = 2,
    TokenNotFound = 3,
    InvalidClaimData = 4,
    ReceiptNotFound = 5,
    NoTokensProvided = 6,
    NotTokenOwner = 7,
    RecTokenNotSet = 8,
    DuplicateToken = 9,
    ContractPaused = 10,
    VerifierAlreadySet = 11,
    VerifierNotFound = 12,
}
