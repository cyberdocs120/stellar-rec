use soroban_sdk::{symbol_short, Address, Bytes, Env, Symbol};

use crate::types::Order;

pub fn admin_key() -> Symbol {
    symbol_short!("Admin")
}

pub fn fee_rate_key() -> Symbol {
    symbol_short!("FeeRt")
}

pub fn order_counter_key() -> Symbol {
    symbol_short!("OrCN")
}

pub fn rec_token_key() -> Symbol {
    symbol_short!("RecT")
}

pub fn usdc_token_key() -> Symbol {
    symbol_short!("USDC")
}

pub fn fee_vault_key() -> Symbol {
    symbol_short!("FVal")
}

pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&admin_key(), admin);
}

pub fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&admin_key()).unwrap()
}

pub fn write_fee_rate(env: &Env, rate: u32) {
    env.storage().instance().set(&fee_rate_key(), &rate);
}

pub fn read_fee_rate(env: &Env) -> u32 {
    env.storage().instance().get(&fee_rate_key()).unwrap_or(10)
}

pub fn write_order_counter(env: &Env, counter: u64) {
    env.storage().instance().set(&order_counter_key(), &counter);
}

pub fn read_order_counter(env: &Env) -> u64 {
    env.storage().instance().get(&order_counter_key()).unwrap_or(0)
}

pub fn write_rec_token(env: &Env, addr: &Address) {
    env.storage().instance().set(&rec_token_key(), addr);
}

pub fn read_rec_token(env: &Env) -> Address {
    env.storage().instance().get(&rec_token_key()).unwrap()
}

pub fn write_usdc_token(env: &Env, addr: &Address) {
    env.storage().instance().set(&usdc_token_key(), addr);
}

pub fn read_usdc_token(env: &Env) -> Address {
    env.storage().instance().get(&usdc_token_key()).unwrap()
}

pub fn write_fee_vault(env: &Env, addr: &Address) {
    env.storage().instance().set(&fee_vault_key(), addr);
}

pub fn read_fee_vault(env: &Env) -> Address {
    env.storage().instance().get(&fee_vault_key()).unwrap()
}

pub fn order_storage_key(env: &Env, order_id: u64) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(0x10);
    for b in order_id.to_be_bytes() {
        key.push_back(b);
    }
    key
}

pub fn write_order(env: &Env, order_id: u64, order: &Order) {
    let key = order_storage_key(env, order_id);
    env.storage().persistent().set(&key, order);
}

pub fn read_order(env: &Env, order_id: u64) -> Order {
    let key = order_storage_key(env, order_id);
    env.storage().persistent().get(&key).unwrap()
}

pub fn has_order(env: &Env, order_id: u64) -> bool {
    let key = order_storage_key(env, order_id);
    env.storage().persistent().has(&key)
}
