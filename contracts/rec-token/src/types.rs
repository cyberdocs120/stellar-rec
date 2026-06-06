use soroban_sdk::{contracttype, Address, Bytes, BytesN};

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum FuelType {
    Solar,
    Wind,
    Hydro,
    Other,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum RecState {
    Active,
    Retired,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct RecMetadata {
    pub token_id: u64,
    pub asset_id: BytesN<32>,
    pub generation_timestamp: u64,
    pub fuel_type: FuelType,
    pub vintage_year: u32,
    pub metadata_uri: Bytes,
    pub state: RecState,
    pub retired_at: Option<u64>,
    pub retirement_receipt: Option<BytesN<32>>,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct RecTokenValue {
    pub owner: Address,
    pub metadata: RecMetadata,
}
