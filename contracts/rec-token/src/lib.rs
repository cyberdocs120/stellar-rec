#![no_std]

mod storage;
mod types;
mod errors;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, Address, Bytes, BytesN, Env};

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

    // ---------- Mint ----------

    pub fn mint(
        env: Env,
        to: Address,
        asset_id: BytesN<32>,
        generation_timestamp: u64,
        fuel_type: FuelType,
        vintage_year: u32,
        metadata_uri: Bytes,
    ) -> u64 {
        let minter = read_authorized_minter(&env);
        minter.require_auth();

        let token_id = read_token_id_counter(&env) + 1;
        write_token_id_counter(&env, token_id);

        let metadata = RecMetadata {
            token_id,
            asset_id,
            generation_timestamp,
            fuel_type,
            vintage_year,
            metadata_uri,
            state: RecState::Active,
            retired_at: None,
            retirement_receipt: None,
        };

        let token = RecTokenValue {
            owner: to.clone(),
            metadata,
        };
        write_token(&env, token_id, &token);

        let count = read_owner_count(&env, &to) + 1;
        write_owner_count(&env, &to, count);

        let supply = read_total_supply(&env) + 1;
        write_total_supply(&env, supply);

        env.events().publish(
            (symbol_short!("rec"), symbol_short!("mint")),
            (token_id, generation_timestamp, 1u64),
        );

        token_id
    }

    // ---------- Transfer ----------

    pub fn transfer(env: Env, from: Address, to: Address, token_id: u64) {
        from.require_auth();

        if !has_token(&env, token_id) {
            panic_with_error!(&env, RecTokenError::TokenNotFound);
        }

        let mut token = read_token(&env, token_id);
        if token.owner != from {
            panic_with_error!(&env, RecTokenError::Unauthorized);
        }
        if token.metadata.state != RecState::Active {
            panic_with_error!(&env, RecTokenError::RecAlreadyRetired);
        }

        token.owner = to.clone();
        write_token(&env, token_id, &token);

        let from_count = read_owner_count(&env, &from);
        write_owner_count(&env, &from, from_count.saturating_sub(1));

        let to_count = read_owner_count(&env, &to) + 1;
        write_owner_count(&env, &to, to_count);

        env.events().publish(
            (symbol_short!("rec"), symbol_short!("xfer")),
            (token_id, from, to),
        );
    }

    // ---------- Burn ----------

    pub fn burn(env: Env, caller: Address, token_id: u64) {
        caller.require_auth();

        if !has_token(&env, token_id) {
            panic_with_error!(&env, RecTokenError::TokenNotFound);
        }

        let mut token = read_token(&env, token_id);
        if token.metadata.state != RecState::Active {
            panic_with_error!(&env, RecTokenError::RecAlreadyRetired);
        }

        token.metadata.state = RecState::Retired;
        token.metadata.retired_at = Some(env.ledger().timestamp());
        write_token(&env, token_id, &token);

        let count = read_owner_count(&env, &token.owner);
        write_owner_count(&env, &token.owner, count.saturating_sub(1));

        let supply = read_total_supply(&env);
        write_total_supply(&env, supply.saturating_sub(1));

        env.events().publish(
            (symbol_short!("rec"), symbol_short!("burn")),
            (token_id, caller),
        );
    }
}
