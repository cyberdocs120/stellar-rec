use soroban_sdk::{contracttype, Address, Bytes, BytesN, Vec};

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
    pub operator: Address,
    pub active: bool,
    pub registered_at: u64,
    pub stake: i128,
    pub rewards: i128,
    pub reputation: u32,
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
    pub oracles: Vec<BytesN<32>>,
}
