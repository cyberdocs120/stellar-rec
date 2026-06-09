use soroban_sdk::{contracttype, Address};

#[derive(Copy, Clone, Debug, PartialEq)]
#[contracttype]
pub enum OrderSide {
    Buy = 0,
    Sell = 1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[contracttype]
pub enum OrderRestriction {
    None = 0,
    FillOrKill = 1,
    ImmediateOrCancel = 2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[contracttype]
pub enum OrderStatus {
    Open = 0,
    Filled = 1,
    Cancelled = 2,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Order {
    pub order_id: u64,
    pub trader: Address,
    pub side: OrderSide,
    pub price: i128,
    pub initial_qty: u64,
    pub remaining_qty: u64,
    pub timestamp: u64,
    pub restrictions: OrderRestriction,
    pub vintage_filter: Option<u32>,
    pub status: OrderStatus,
}
