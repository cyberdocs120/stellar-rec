#![no_std]

mod storage;
mod types;
mod errors;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, Address, Bytes, BytesN, Env};

use errors::OracleError;
use storage::*;
use types::*;

#[contract]
pub struct OracleHandlerContract;

#[contractimpl]
impl OracleHandlerContract {
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        write_admin(&env, &admin);
        write_paused(&env, false);
        write_threshold_n(&env, 1);
        write_threshold_d(&env, 1);
        write_oracle_count(&env, 0);
    }

    pub fn admin(env: Env) -> Address {
        read_admin(&env)
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        new_admin.require_auth();
        write_admin(&env, &new_admin);
    }

    pub fn set_rec_token(env: Env, addr: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_rec_token(&env, &addr);
    }

    pub fn rec_token(env: Env) -> Address {
        read_rec_token(&env)
    }

    pub fn register_oracle(env: Env, pubkey: BytesN<32>, uri: Bytes) {
        let admin = read_admin(&env);
        admin.require_auth();

        if has_oracle(&env, &pubkey) {
            panic_with_error!(&env, OracleError::OracleAlreadyRegistered);
        }

        let node = OracleNode {
            pubkey: pubkey.clone(),
            uri,
            active: true,
            registered_at: env.ledger().timestamp(),
        };
        write_oracle(&env, &pubkey, &node);

        let count = read_oracle_count(&env) + 1;
        write_oracle_count(&env, count);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("reg")),
            (pubkey,),
        );
    }

    pub fn revoke_oracle(env: Env, pubkey: BytesN<32>) {
        let admin = read_admin(&env);
        admin.require_auth();

        if !has_oracle(&env, &pubkey) {
            panic_with_error!(&env, OracleError::OracleNotFound);
        }

        let mut node = read_oracle(&env, &pubkey);
        node.active = false;
        write_oracle(&env, &pubkey, &node);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("rev")),
            (pubkey,),
        );
    }

    pub fn oracle_count(env: Env) -> u32 {
        read_oracle_count(&env)
    }

    pub fn get_oracle(env: Env, pubkey: BytesN<32>) -> OracleNode {
        if !has_oracle(&env, &pubkey) {
            panic_with_error!(&env, OracleError::OracleNotFound);
        }
        read_oracle(&env, &pubkey)
    }

    pub fn set_threshold(env: Env, n: u32, d: u32) {
        let admin = read_admin(&env);
        admin.require_auth();

        if n == 0 || d == 0 || n > d {
            panic_with_error!(&env, OracleError::Unauthorized);
        }

        write_threshold_n(&env, n);
        write_threshold_d(&env, d);
    }

    pub fn threshold(env: Env) -> (u32, u32) {
        (read_threshold_n(&env), read_threshold_d(&env))
    }

    pub fn set_meter(env: Env, meter_id: BytesN<32>, asset_id: BytesN<32>, capacity_mw: u64) {
        let admin = read_admin(&env);
        admin.require_auth();

        let binding = MeterBinding {
            meter_id: meter_id.clone(),
            asset_id,
            capacity_mw,
        };
        write_meter(&env, &meter_id, &binding);
    }

    pub fn get_meter(env: Env, meter_id: BytesN<32>) -> MeterBinding {
        if !has_meter(&env, &meter_id) {
            panic_with_error!(&env, OracleError::MeterNotBound);
        }
        read_meter(&env, &meter_id)
    }

    pub fn pause(env: Env) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_paused(&env, true);
        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("pause")),
            (),
        );
    }

    pub fn resume(env: Env) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_paused(&env, false);
        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("resume")),
            (),
        );
    }

    pub fn paused(env: Env) -> bool {
        read_paused(&env)
    }

    // ---------- Stubs for Day 6 ----------

    pub fn submit_reading(
        _env: Env,
        _meter_id: BytesN<32>,
        _mwh: u64,
        _generation_timestamp: u64,
        _signatures: soroban_sdk::Vec<(BytesN<32>, BytesN<64>)>,
        _fuel_type: u32,
        _vintage_year: u32,
        _metadata_uri: Bytes,
    ) -> u64 {
        panic_with_error!(&_env, OracleError::Unauthorized);
    }

    pub fn dispute(_env: Env, _reading_hash: BytesN<32>) {
        panic_with_error!(&_env, OracleError::Unauthorized);
    }

    pub fn resolve_dispute(_env: Env, _reading_hash: BytesN<32>, _outcome: bool) {
        panic_with_error!(&_env, OracleError::Unauthorized);
    }
}
