use soroban_sdk::{symbol_short, xdr::ToXdr, Address, Bytes, Env, Symbol};

use crate::types::RecTokenValue;

// ---------- Singleton key helpers ----------

pub fn admin_key() -> Symbol {
    symbol_short!("Admin")
}

pub fn token_id_counter_key() -> Symbol {
    symbol_short!("TIDC")
}

pub fn total_supply_key() -> Symbol {
    symbol_short!("TSup")
}

pub fn authorized_minter_key() -> Symbol {
    symbol_short!("AMnt")
}

pub fn authorized_burner_key() -> Symbol {
    symbol_short!("ABrn")
}

pub fn paused_key() -> Symbol {
    symbol_short!("Paus")
}

// ---------- Admin ----------

pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&admin_key(), admin);
}

pub fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&admin_key()).unwrap()
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&admin_key())
}

// ---------- Token ID counter ----------

pub fn write_token_id_counter(env: &Env, counter: u64) {
    env.storage()
        .instance()
        .set(&token_id_counter_key(), &counter);
}

pub fn read_token_id_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&token_id_counter_key())
        .unwrap_or(0)
}

// ---------- Total supply ----------

pub fn write_total_supply(env: &Env, supply: u64) {
    env.storage().instance().set(&total_supply_key(), &supply);
}

pub fn read_total_supply(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&total_supply_key())
        .unwrap_or(0)
}

// ---------- Authorized minter ----------

pub fn write_authorized_minter(env: &Env, minter: &Address) {
    env.storage()
        .instance()
        .set(&authorized_minter_key(), minter);
}

pub fn read_authorized_minter(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&authorized_minter_key())
        .unwrap()
}

pub fn has_authorized_minter(env: &Env) -> bool {
    env.storage().instance().has(&authorized_minter_key())
}

// ---------- Authorized burner ----------

pub fn write_authorized_burner(env: &Env, burner: &Address) {
    env.storage()
        .instance()
        .set(&authorized_burner_key(), burner);
}

pub fn read_authorized_burner(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&authorized_burner_key())
        .unwrap()
}

pub fn has_authorized_burner(env: &Env) -> bool {
    env.storage().instance().has(&authorized_burner_key())
}

pub fn write_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&paused_key(), &paused);
}

pub fn read_paused(env: &Env) -> bool {
    env.storage().instance().get(&paused_key()).unwrap_or(false)
}

// ---------- Per-token storage ----------

pub fn token_storage_key(env: &Env, token_id: u64) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(0x01);
    for b in token_id.to_be_bytes() {
        key.push_back(b);
    }
    key
}

pub fn write_token(env: &Env, token_id: u64, token: &RecTokenValue) {
    let key = token_storage_key(env, token_id);
    env.storage().persistent().set(&key, token);
}

pub fn read_token(env: &Env, token_id: u64) -> RecTokenValue {
    let key = token_storage_key(env, token_id);
    env.storage().persistent().get(&key).unwrap()
}

pub fn has_token(env: &Env, token_id: u64) -> bool {
    let key = token_storage_key(env, token_id);
    env.storage().persistent().has(&key)
}

// ---------- Per-owner token count ----------

pub fn owner_count_key(env: &Env, owner: &Address) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(0x02);
    let owner_bytes = owner.to_xdr(env);
    for i in 0..owner_bytes.len() {
        key.push_back(owner_bytes.get(i).unwrap());
    }
    key
}

pub fn write_owner_count(env: &Env, owner: &Address, count: u64) {
    let key = owner_count_key(env, owner);
    env.storage().persistent().set(&key, &count);
}

pub fn read_owner_count(env: &Env, owner: &Address) -> u64 {
    let key = owner_count_key(env, owner);
    env.storage().persistent().get(&key).unwrap_or(0)
}
