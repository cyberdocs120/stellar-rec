use soroban_sdk::{contracttype, Address, Bytes, BytesN};

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum FuelType {
    Solar,
    Wind,
    Hydro,
    Other,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct OracleNode {
    pub pubkey: BytesN<32>,
    pub uri: Bytes,
    pub active: bool,
    pub registered_at: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MeterBinding {
    pub meter_id: BytesN<32>,
    pub asset_id: BytesN<32>,
    pub receiver: Address,
    pub capacity_mw: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ReadingRecord {
    pub reading_hash: BytesN<32>,
    pub meter_id: BytesN<32>,
    pub mwh: u64,
    pub timestamp: u64,
    pub oracle_count: u32,
    pub threshold_met: bool,
    pub disputed: bool,
    pub resolved: bool,
    pub token_id: Option<u64>,
}
