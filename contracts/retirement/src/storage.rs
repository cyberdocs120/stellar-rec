use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, Map, Symbol};

use crate::types::RetirementReceipt;

pub fn admin_key() -> Symbol {
    symbol_short!("Admin")
}

pub fn rec_token_key() -> Symbol {
    symbol_short!("RecT")
}

pub fn receipt_counter_key() -> Symbol {
    symbol_short!("RetCN")
}

pub fn retirement_fee_key() -> Symbol {
    symbol_short!("RetFee")
}

pub fn usdc_token_key() -> Symbol {
    symbol_short!("USDC")
}

pub fn fee_vault_key() -> Symbol {
    symbol_short!("FVal")
}

pub fn paused_key() -> Symbol {
    symbol_short!("Paus")
}

pub fn verifiers_key() -> Symbol {
    symbol_short!("Verif")
}

pub fn write_verifiers(env: &Env, verifiers: &Map<Address, bool>) {
    env.storage().instance().set(&verifiers_key(), verifiers);
}

pub fn read_verifiers(env: &Env) -> Map<Address, bool> {
    env.storage()
        .instance()
        .get(&verifiers_key())
        .unwrap_or(Map::new(env))
}

pub fn has_verifier(env: &Env, verifier: &Address) -> bool {
    let verifiers = read_verifiers(env);
    verifiers.contains_key(verifier.clone()) && verifiers.get(verifier.clone()).unwrap()
}

pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&admin_key(), admin);
}

pub fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&admin_key()).unwrap()
}

pub fn write_rec_token(env: &Env, addr: &Address) {
    env.storage().instance().set(&rec_token_key(), addr);
}

pub fn read_rec_token(env: &Env) -> Address {
    env.storage().instance().get(&rec_token_key()).unwrap()
}

pub fn has_rec_token(env: &Env) -> bool {
    env.storage().instance().has(&rec_token_key())
}

pub fn write_receipt_counter(env: &Env, counter: u64) {
    env.storage()
        .instance()
        .set(&receipt_counter_key(), &counter);
}

pub fn read_receipt_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&receipt_counter_key())
        .unwrap_or(0)
}

pub fn write_retirement_fee(env: &Env, fee: i128) {
    env.storage().instance().set(&retirement_fee_key(), &fee);
}

pub fn read_retirement_fee(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&retirement_fee_key())
        .unwrap_or(2_000_000i128)
}

pub fn write_usdc_token(env: &Env, addr: &Address) {
    env.storage().instance().set(&usdc_token_key(), addr);
}

pub fn read_usdc_token(env: &Env) -> Address {
    env.storage().instance().get(&usdc_token_key()).unwrap()
}

pub fn has_usdc_token(env: &Env) -> bool {
    env.storage().instance().has(&usdc_token_key())
}

pub fn write_fee_vault(env: &Env, addr: &Address) {
    env.storage().instance().set(&fee_vault_key(), addr);
}

pub fn read_fee_vault(env: &Env) -> Address {
    env.storage().instance().get(&fee_vault_key()).unwrap()
}

pub fn has_fee_vault(env: &Env) -> bool {
    env.storage().instance().has(&fee_vault_key())
}

pub fn write_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&paused_key(), &paused);
}

pub fn read_paused(env: &Env) -> bool {
    env.storage().instance().get(&paused_key()).unwrap_or(false)
}

pub fn receipt_storage_key(env: &Env, receipt_id: &BytesN<32>) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(0x01);
    for i in 0..32 {
        key.push_back(receipt_id.get(i).unwrap());
    }
    key
}

pub fn write_receipt(env: &Env, receipt_id: &BytesN<32>, receipt: &RetirementReceipt) {
    let key = receipt_storage_key(env, receipt_id);
    env.storage().persistent().set(&key, receipt);
}

pub fn read_receipt(env: &Env, receipt_id: &BytesN<32>) -> RetirementReceipt {
    let key = receipt_storage_key(env, receipt_id);
    env.storage().persistent().get(&key).unwrap()
}

pub fn has_receipt(env: &Env, receipt_id: &BytesN<32>) -> bool {
    let key = receipt_storage_key(env, receipt_id);
    env.storage().persistent().has(&key)
}

pub fn token_retirement_key(env: &Env, token_id: u64) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(0x02);
    for b in token_id.to_be_bytes() {
        key.push_back(b);
    }
    key
}

pub fn write_retired_token(env: &Env, token_id: u64, receipt_id: &BytesN<32>) {
    let key = token_retirement_key(env, token_id);
    env.storage().persistent().set(&key, receipt_id);
}

pub fn read_retired_token(env: &Env, token_id: u64) -> Option<BytesN<32>> {
    let key = token_retirement_key(env, token_id);
    env.storage().persistent().get(&key)
}

pub fn has_retired_token(env: &Env, token_id: u64) -> bool {
    let key = token_retirement_key(env, token_id);
    env.storage().persistent().has(&key)
}
