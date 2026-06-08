#![no_std]

mod storage;
mod types;
mod errors;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, Address, Bytes, BytesN, Env, IntoVal};

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

    pub fn set_meter(env: Env, meter_id: BytesN<32>, receiver: Address, asset_id: BytesN<32>, capacity_mw: u64) {
        let admin = read_admin(&env);
        admin.require_auth();

        let binding = MeterBinding {
            meter_id: meter_id.clone(),
            asset_id,
            receiver,
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

    pub fn submit_reading(
        env: Env,
        meter_id: BytesN<32>,
        mwh: u64,
        generation_timestamp: u64,
        signatures: soroban_sdk::Vec<(BytesN<32>, BytesN<64>)>,
        fuel_type: u32,
        vintage_year: u32,
        metadata_uri: Bytes,
    ) -> u64 {
        if read_paused(&env) {
            panic_with_error!(&env, OracleError::ContractPaused);
        }

        if !has_meter(&env, &meter_id) {
            panic_with_error!(&env, OracleError::MeterNotBound);
        }

        let meter = read_meter(&env, &meter_id);
        if mwh > meter.capacity_mw * 24 {
            panic_with_error!(&env, OracleError::InvalidMeterReading);
        }

        let n = read_threshold_n(&env);
        if signatures.len() < n {
            panic_with_error!(&env, OracleError::ThresholdNotMet);
        }

        let mut data = Bytes::new(&env);
        data.append(&meter_id.clone().into());
        data.append(&Bytes::from_slice(&env, &mwh.to_be_bytes()));
        data.append(&Bytes::from_slice(&env, &generation_timestamp.to_be_bytes()));

        let reading_hash: BytesN<32> = env.crypto().sha256(&data).into();

        if has_reading(&env, &reading_hash) {
            panic_with_error!(&env, OracleError::Unauthorized); // Prevent double submission
        }

        for i in 0..signatures.len() {
            let (pubkey, sig) = signatures.get(i).unwrap();
            if !has_oracle(&env, &pubkey) {
                panic_with_error!(&env, OracleError::InvalidSignature);
            }
            let oracle = read_oracle(&env, &pubkey);
            if !oracle.active {
                panic_with_error!(&env, OracleError::InvalidSignature);
            }
            env.crypto().ed25519_verify(&pubkey, &data, &sig);
        }

        let fuel_enum = match fuel_type {
            0 => FuelType::Solar,
            1 => FuelType::Wind,
            2 => FuelType::Hydro,
            _ => FuelType::Other,
        };

        let rec_token_addr = read_rec_token(&env);
        let token_id: u64 = env.invoke_contract(
            &rec_token_addr,
            &symbol_short!("mint"),
            (
                meter.receiver,
                meter.asset_id,
                generation_timestamp,
                fuel_enum,
                vintage_year,
                metadata_uri,
            )
            .into_val(&env),
        );

        let record = ReadingRecord {
            reading_hash: reading_hash.clone(),
            meter_id: meter_id.clone(),
            mwh,
            timestamp: generation_timestamp,
            oracle_count: signatures.len() as u32,
            threshold_met: true,
            disputed: false,
            resolved: false,
            token_id: Some(token_id),
        };
        write_reading(&env, &reading_hash, &record);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("read")),
            (reading_hash, meter_id, mwh, signatures.len() as u32, token_id),
        );

        token_id
    }

    pub fn dispute(env: Env, reading_hash: BytesN<32>) {
        if !has_reading(&env, &reading_hash) {
            panic_with_error!(&env, OracleError::OracleNotFound); // Using OracleNotFound as a generic "Not Found"
        }

        let mut record = read_reading(&env, &reading_hash);
        if record.resolved {
            panic_with_error!(&env, OracleError::AlreadyResolved);
        }

        record.disputed = true;
        write_reading(&env, &reading_hash, &record);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("disp")),
            (reading_hash,),
        );
    }

    pub fn resolve_dispute(env: Env, reading_hash: BytesN<32>, outcome: bool) {
        let admin = read_admin(&env);
        admin.require_auth();

        if !has_reading(&env, &reading_hash) {
            panic_with_error!(&env, OracleError::OracleNotFound);
        }

        let mut record = read_reading(&env, &reading_hash);
        if record.resolved {
            panic_with_error!(&env, OracleError::AlreadyResolved);
        }

        record.resolved = true;
        // Outcome could involve slashing or other logic, but for now we just mark as resolved.
        write_reading(&env, &reading_hash, &record);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("resd")),
            (reading_hash, outcome),
        );
    }
}
