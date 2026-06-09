use soroban_sdk::contracterror;

#[derive(Copy, Clone, Debug, PartialEq)]
#[contracterror]
pub enum MarketError {
    Unauthorized = 1,
    OrderNotFound = 2,
    OrderFilled = 3,
    PriceMismatch = 4,
    InsufficientBalance = 5,
    FeeCapExceeded = 6,
    VintageMismatch = 7,
    InvalidQuantity = 8,
}
