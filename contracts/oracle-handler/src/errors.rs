use soroban_sdk::contracterror;

#[derive(Copy, Clone, Debug, PartialEq)]
#[contracterror]
pub enum OracleError {
    Unauthorized = 1,
    OracleAlreadyRegistered = 2,
    OracleNotFound = 3,
    ThresholdNotMet = 4,
    InvalidSignature = 5,
    InvalidMeterReading = 6,
    MeterNotBound = 7,
    AlreadyResolved = 8,
    DisputeWindowExpired = 9,
    ContractPaused = 10,
}
