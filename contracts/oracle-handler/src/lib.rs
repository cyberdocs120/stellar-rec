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
        write_minimum_bond(&env, &100_000_000_000); // 100 yUSDC default
        write_reward_per_reading(&env, &1_000_000); // 0.01 yUSDC default
        write_reward_pool(&env, &0);
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

    pub fn register_oracle(env: Env, pubkey: BytesN<32>, uri: Bytes, operator: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        if has_oracle(&env, &pubkey) {
            panic_with_error!(&env, OracleError::OracleAlreadyRegistered);
        }

        let node = OracleNode {
            pubkey: pubkey.clone(),
            uri,
            operator: operator.clone(),
            active: false,
            registered_at: env.ledger().timestamp(),
            stake: 0,
            rewards: 0,
            reputation: 0,
        };
        write_oracle(&env, &pubkey, &node);

        let count = read_oracle_count(&env) + 1;
        write_oracle_count(&env, count);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("reg")),
            (pubkey, operator),
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

    // ---------- USDC Token ----------

    pub fn set_usdc_token(env: Env, addr: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_usdc_token(&env, &addr);
    }

    pub fn usdc_token(env: Env) -> Address {
        read_usdc_token(&env)
    }

    // ---------- Minimum Bond ----------

    pub fn set_minimum_bond(env: Env, amount: i128) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_minimum_bond(&env, &amount);
    }

    pub fn minimum_bond(env: Env) -> i128 {
        read_minimum_bond(&env)
    }

    // ---------- Oracle Staking ----------

    pub fn deposit_bond(env: Env, pubkey: BytesN<32>, amount: i128) {
        if !has_oracle(&env, &pubkey) {
            panic_with_error!(&env, OracleError::OracleNotFound);
        }

        let mut node = read_oracle(&env, &pubkey);
        node.operator.require_auth();

        let min_bond = read_minimum_bond(&env);
        let usdc_token = read_usdc_token(&env);

        // Transfer yUSDC from the operator to this contract as bond
        let _: () = env.invoke_contract(
            &usdc_token,
            &symbol_short!("xfer"),
            (node.operator.clone(), env.current_contract_address(), amount).into_val(&env),
        );

        node.stake += amount;
        if node.stake >= min_bond && !node.active {
            node.active = true;
        }
        write_oracle(&env, &pubkey, &node);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("bond")),
            (pubkey, node.stake, node.active),
        );
    }

    pub fn withdraw_bond(env: Env, pubkey: BytesN<32>, amount: i128) {
        if !has_oracle(&env, &pubkey) {
            panic_with_error!(&env, OracleError::OracleNotFound);
        }

        let mut node = read_oracle(&env, &pubkey);
        node.operator.require_auth();

        if amount > node.stake {
            panic_with_error!(&env, OracleError::InsufficientBond);
        }

        let usdc_token = read_usdc_token(&env);
        let _: () = env.invoke_contract(
            &usdc_token,
            &symbol_short!("xfer"),
            (env.current_contract_address(), node.operator.clone(), amount).into_val(&env),
        );

        node.stake -= amount;
        let min_bond = read_minimum_bond(&env);
        if node.stake < min_bond && node.active {
            node.active = false;
        }
        write_oracle(&env, &pubkey, &node);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("unbd")),
            (pubkey, node.stake),
        );
    }

    // ---------- Reward Per Reading ----------

    pub fn set_reward_per_reading(env: Env, amount: i128) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_reward_per_reading(&env, &amount);
    }

    pub fn reward_per_reading(env: Env) -> i128 {
        read_reward_per_reading(&env)
    }

    // ---------- Reward Pool ----------

    pub fn fund_reward_pool(env: Env, amount: i128) {
        let admin = read_admin(&env);
        admin.require_auth();

        let usdc_token = read_usdc_token(&env);
        let _: () = env.invoke_contract(
            &usdc_token,
            &symbol_short!("xfer"),
            (admin.clone(), env.current_contract_address(), amount).into_val(&env),
        );

        let pool = read_reward_pool(&env) + amount;
        write_reward_pool(&env, &pool);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("fund")),
            (amount, pool),
        );
    }

    pub fn reward_pool(env: Env) -> i128 {
        read_reward_pool(&env)
    }

    pub fn set_price(env: Env, price: i128) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_price(&env, price);
        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("prce")),
            (price,),
        );
    }

    pub fn get_price(env: Env) -> i128 {
        read_price(&env)
    }

    pub fn claim_rewards(env: Env, pubkey: BytesN<32>) {
        if !has_oracle(&env, &pubkey) {
            panic_with_error!(&env, OracleError::OracleNotFound);
        }

        let mut node = read_oracle(&env, &pubkey);
        node.operator.require_auth();

        if node.rewards <= 0 {
            panic_with_error!(&env, OracleError::NoRewardsToClaim);
        }

        let amount = node.rewards;
        node.rewards = 0;
        write_oracle(&env, &pubkey, &node);

        let usdc_token = read_usdc_token(&env);
        let _: () = env.invoke_contract(
            &usdc_token,
            &symbol_short!("xfer"),
            (env.current_contract_address(), node.operator.clone(), amount).into_val(&env),
        );

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("rewd")),
            (pubkey, amount),
        );
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

        let min_bond = read_minimum_bond(&env);
        let mut oracle_list: soroban_sdk::Vec<BytesN<32>> = soroban_sdk::Vec::new(&env);
        for i in 0..signatures.len() {
            let (pubkey, sig) = signatures.get(i).unwrap();
            if !has_oracle(&env, &pubkey) {
                panic_with_error!(&env, OracleError::InvalidSignature);
            }
            let oracle = read_oracle(&env, &pubkey);
            if !oracle.active {
                panic_with_error!(&env, OracleError::InvalidSignature);
            }
            if oracle.stake < min_bond {
                panic_with_error!(&env, OracleError::BondTooLow);
            }
            env.crypto().ed25519_verify(&pubkey, &data, &sig);
            oracle_list.push_back(pubkey);
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
            oracles: oracle_list,
        };
        write_reading(&env, &reading_hash, &record);

        // Distribute rewards to participating oracles
        let reward_per = read_reward_per_reading(&env);
        let mut pool = read_reward_pool(&env);
        let num_oracles = signatures.len() as i128;
        let total_reward = reward_per * num_oracles;
        if pool >= total_reward && num_oracles > 0 {
            pool -= total_reward;
            write_reward_pool(&env, &pool);
            for i in 0..signatures.len() {
                let (pubkey, _) = signatures.get(i).unwrap();
                let mut oracle = read_oracle(&env, &pubkey);
                oracle.rewards += reward_per;
                oracle.reputation = oracle.reputation.saturating_add(1);
                write_oracle(&env, &pubkey, &oracle);
            }
        }

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

        // If outcome is false (reading was fraudulent), slash all signing oracles
        if !outcome {
            let usdc_token = read_usdc_token(&env);
            let admin = read_admin(&env);
            for i in 0..record.oracles.len() {
                let pubkey = record.oracles.get(i).unwrap();
                let mut oracle = read_oracle(&env, &pubkey);
                if oracle.stake > 0 {
                    let slash_amount = oracle.stake;
                    oracle.stake = 0;
                    // Transfer slashed stake to admin as penalty
                    let _: () = env.invoke_contract(
                        &usdc_token,
                        &symbol_short!("xfer"),
                        (env.current_contract_address(), admin.clone(), slash_amount).into_val(&env),
                    );
                }
                oracle.active = false;
                oracle.reputation = oracle.reputation.saturating_sub(10);
                write_oracle(&env, &pubkey, &oracle);
            }
        } else {
            // Reading was valid — boost reputation of honest oracles
            for i in 0..record.oracles.len() {
                let pubkey = record.oracles.get(i).unwrap();
                let mut oracle = read_oracle(&env, &pubkey);
                oracle.reputation = oracle.reputation.saturating_add(5);
                write_oracle(&env, &pubkey, &oracle);
            }
        }

        write_reading(&env, &reading_hash, &record);

        env.events().publish(
            (symbol_short!("orcl"), symbol_short!("resd")),
            (reading_hash, outcome),
        );
    }
}
