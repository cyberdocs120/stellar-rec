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

#[derive(Copy, Clone, Debug, PartialEq)]
#[contracttype]
pub enum PositionState {
    Pending = 0,
    Active = 1,
    Settled = 2,
    Expired = 3,
    Liquidated = 4,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct CfDPosition {
    pub position_id: u64,
    pub counterparty_a: Address,
    pub counterparty_b: Option<Address>,
    pub strike_price: i128,
    pub quantity: u64,
    pub settlement_date: u64,
    pub collateral_a: i128,
    pub collateral_b: i128,
    pub maintenance_margin_bps: u32,
    pub state: PositionState,
    pub last_mtm_timestamp: u64,
    pub mtm_value: i128,
}
