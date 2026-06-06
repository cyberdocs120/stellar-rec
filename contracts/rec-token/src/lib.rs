#![no_std]

mod storage;
mod types;
mod errors;

use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env};

use errors::RecTokenError;
use storage::*;
use types::*;

#[contract]
pub struct RecTokenContract;

#[contractimpl]
impl RecTokenContract {
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        write_admin(&env, &admin);
        write_token_id_counter(&env, 0);
        write_total_supply(&env, 0);
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

    pub fn total_supply(env: Env) -> u64 {
        read_total_supply(&env)
    }

    pub fn balance_of(env: Env, owner: Address) -> u64 {
        read_owner_count(&env, &owner)
    }

    pub fn owner_of(env: Env, token_id: u64) -> Address {
        if !has_token(&env, token_id) {
            panic_with_error!(&env, RecTokenError::TokenNotFound);
        }
        let token = read_token(&env, token_id);
        token.owner
    }

    pub fn token_uri(env: Env, token_id: u64) -> Bytes {
        if !has_token(&env, token_id) {
            panic_with_error!(&env, RecTokenError::TokenNotFound);
        }
        let token = read_token(&env, token_id);
        token.metadata.metadata_uri
    }

    pub fn token_metadata(env: Env, token_id: u64) -> RecMetadata {
        if !has_token(&env, token_id) {
            panic_with_error!(&env, RecTokenError::TokenNotFound);
        }
        let token = read_token(&env, token_id);
        token.metadata
    }

    pub fn authorize_minter(env: Env, minter: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_authorized_minter(&env, &minter);
    }

    pub fn authorize_burner(env: Env, burner: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_authorized_burner(&env, &burner);
    }

    pub fn revoke_minter(env: Env) {
        let admin = read_admin(&env);
        admin.require_auth();
        env.storage().instance().remove(&authorized_minter_key());
    }

    pub fn revoke_burner(env: Env) {
        let admin = read_admin(&env);
        admin.require_auth();
        env.storage().instance().remove(&authorized_burner_key());
    }

    // ---------- STUBS (Day 3) ----------

    pub fn mint(
        _env: Env,
        _to: Address,
        _asset_id: BytesN<32>,
        _generation_timestamp: u64,
        _fuel_type: FuelType,
        _vintage_year: u32,
        _metadata_uri: Bytes,
    ) -> u64 {
        panic!("not implemented")
    }

    pub fn transfer(_env: Env, _from: Address, _to: Address, _token_id: u64) {
        panic!("not implemented")
    }

    pub fn burn(_env: Env, _token_id: u64) {
        panic!("not implemented")
    }
}
