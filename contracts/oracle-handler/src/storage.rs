use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, Symbol};

use crate::types::{MeterBinding, OracleNode, ReadingRecord};

// ---------- Singleton key helpers ----------

pub fn admin_key() -> Symbol {
    symbol_short!("Admin")
}

pub fn pause_key() -> Symbol {
    symbol_short!("Pause")
}

pub fn threshold_n_key() -> Symbol {
    symbol_short!("ThN")
}

pub fn threshold_d_key() -> Symbol {
    symbol_short!("ThD")
}

pub fn oracle_count_key() -> Symbol {
    symbol_short!("OrC")
}

pub fn rec_token_key() -> Symbol {
    symbol_short!("RecT")
}

pub fn usdc_token_key() -> Symbol {
    symbol_short!("USDCT")
}

pub fn minimum_bond_key() -> Symbol {
    symbol_short!("MinB")
}

pub fn reward_pool_key() -> Symbol {
    symbol_short!("RewP")
}

pub fn reward_per_reading_key() -> Symbol {
    symbol_short!("RewR")
}

pub fn price_key() -> Symbol {
    symbol_short!("Price")
}

// ---------- Admin ----------

pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&admin_key(), admin);
}

pub fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&admin_key()).unwrap()
}

// ---------- Pause ----------

pub fn write_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&pause_key(), &paused);
}

pub fn read_paused(env: &Env) -> bool {
    env.storage().instance().get(&pause_key()).unwrap_or(false)
}

// ---------- Threshold ----------

pub fn write_threshold_n(env: &Env, n: u32) {
    env.storage().instance().set(&threshold_n_key(), &n);
}

pub fn read_threshold_n(env: &Env) -> u32 {
    env.storage().instance().get(&threshold_n_key()).unwrap()
}

pub fn write_threshold_d(env: &Env, d: u32) {
    env.storage().instance().set(&threshold_d_key(), &d);
}

pub fn read_threshold_d(env: &Env) -> u32 {
    env.storage().instance().get(&threshold_d_key()).unwrap()
}

// ---------- Oracle count ----------

pub fn write_oracle_count(env: &Env, count: u32) {
    env.storage().instance().set(&oracle_count_key(), &count);
}

pub fn read_oracle_count(env: &Env) -> u32 {
    env.storage().instance().get(&oracle_count_key()).unwrap_or(0)
}

// ---------- Rec token address ----------

pub fn write_rec_token(env: &Env, addr: &Address) {
    env.storage().instance().set(&rec_token_key(), addr);
}

pub fn read_rec_token(env: &Env) -> Address {
    env.storage().instance().get(&rec_token_key()).unwrap()
}

// ---------- USDC token address ----------

pub fn write_usdc_token(env: &Env, addr: &Address) {
    env.storage().instance().set(&usdc_token_key(), addr);
}

pub fn read_usdc_token(env: &Env) -> Address {
    env.storage().instance().get(&usdc_token_key()).unwrap()
}

// ---------- Minimum bond ----------

pub fn write_minimum_bond(env: &Env, amount: &i128) {
    env.storage().instance().set(&minimum_bond_key(), amount);
}

pub fn read_minimum_bond(env: &Env) -> i128 {
    env.storage().instance().get(&minimum_bond_key()).unwrap_or(100_000_000_000) // 100 yUSDC default
}

// ---------- Reward pool ----------

pub fn write_reward_pool(env: &Env, amount: &i128) {
    env.storage().instance().set(&reward_pool_key(), amount);
}

pub fn read_reward_pool(env: &Env) -> i128 {
    env.storage().instance().get(&reward_pool_key()).unwrap_or(0)
}

// ---------- Reward per reading ----------

pub fn write_reward_per_reading(env: &Env, amount: &i128) {
    env.storage().instance().set(&reward_per_reading_key(), amount);
}

pub fn read_reward_per_reading(env: &Env) -> i128 {
    env.storage().instance().get(&reward_per_reading_key()).unwrap_or(1_000_000) // 0.01 yUSDC default
}

// ---------- Price ----------

pub fn write_price(env: &Env, price: i128) {
    env.storage().instance().set(&price_key(), &price);
}

pub fn read_price(env: &Env) -> i128 {
    env.storage().instance().get(&price_key()).unwrap_or(0)
}

// ---------- Oracle node storage ----------

pub fn oracle_storage_key(env: &Env, pubkey: &BytesN<32>) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(0x10);
    for i in 0..32 {
        key.push_back(pubkey.get(i).unwrap());
    }
    key
}

pub fn write_oracle(env: &Env, pubkey: &BytesN<32>, node: &OracleNode) {
    let key = oracle_storage_key(env, pubkey);
    env.storage().persistent().set(&key, node);
}

pub fn read_oracle(env: &Env, pubkey: &BytesN<32>) -> OracleNode {
    let key = oracle_storage_key(env, pubkey);
    env.storage().persistent().get(&key).unwrap()
}

pub fn has_oracle(env: &Env, pubkey: &BytesN<32>) -> bool {
    let key = oracle_storage_key(env, pubkey);
    env.storage().persistent().has(&key)
}

// ---------- Meter binding storage ----------

pub fn meter_storage_key(env: &Env, meter_id: &BytesN<32>) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(0x20);
    for i in 0..32 {
        key.push_back(meter_id.get(i).unwrap());
    }
    key
}

pub fn write_meter(env: &Env, meter_id: &BytesN<32>, binding: &MeterBinding) {
    let key = meter_storage_key(env, meter_id);
    env.storage().persistent().set(&key, binding);
}

pub fn read_meter(env: &Env, meter_id: &BytesN<32>) -> MeterBinding {
    let key = meter_storage_key(env, meter_id);
    env.storage().persistent().get(&key).unwrap()
}

pub fn has_meter(env: &Env, meter_id: &BytesN<32>) -> bool {
    let key = meter_storage_key(env, meter_id);
    env.storage().persistent().has(&key)
}

// ---------- Reading record storage ----------

pub fn reading_storage_key(env: &Env, reading_hash: &BytesN<32>) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(0x30);
    for i in 0..32 {
        key.push_back(reading_hash.get(i).unwrap());
    }
    key
}

pub fn write_reading(env: &Env, reading_hash: &BytesN<32>, record: &ReadingRecord) {
    let key = reading_storage_key(env, reading_hash);
    env.storage().persistent().set(&key, record);
}

pub fn read_reading(env: &Env, reading_hash: &BytesN<32>) -> ReadingRecord {
    let key = reading_storage_key(env, reading_hash);
    env.storage().persistent().get(&key).unwrap()
}

pub fn has_reading(env: &Env, reading_hash: &BytesN<32>) -> bool {
    let key = reading_storage_key(env, reading_hash);
    env.storage().persistent().has(&key)
}
