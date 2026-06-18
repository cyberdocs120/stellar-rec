#![no_std]

mod errors;
mod storage;
#[cfg(test)]
mod test;
mod types;

use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, BytesN, Env, IntoVal, Symbol,
    Vec,
};

use errors::RetirementError;
use storage::*;
pub use types::*;

#[contract]
pub struct RetirementContract;

#[contractimpl]
impl RetirementContract {
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        write_admin(&env, &admin);
        write_receipt_counter(&env, 0);
        write_retirement_fee(&env, 2_000_000i128);
        write_paused(&env, false);
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

    pub fn set_rec_token(env: Env, id: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_rec_token(&env, &id);
    }

    pub fn rec_token(env: Env) -> Address {
        read_rec_token(&env)
    }

    pub fn set_usdc_token(env: Env, id: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_usdc_token(&env, &id);
    }

    pub fn usdc_token(env: Env) -> Address {
        read_usdc_token(&env)
    }

    pub fn set_fee_vault(env: Env, addr: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_fee_vault(&env, &addr);
    }

    pub fn fee_vault(env: Env) -> Address {
        read_fee_vault(&env)
    }

    pub fn set_retirement_fee(env: Env, fee: i128) {
        let admin = read_admin(&env);
        admin.require_auth();
        if fee < 0 {
            panic_with_error!(&env, RetirementError::InvalidClaimData);
        }
        write_retirement_fee(&env, fee);
    }

    pub fn retirement_fee(env: Env) -> i128 {
        read_retirement_fee(&env)
    }

    pub fn pause(env: Env) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_paused(&env, true);
        env.events()
            .publish((symbol_short!("ret"), symbol_short!("paus")), ());
    }

    pub fn resume(env: Env) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_paused(&env, false);
        env.events()
            .publish((symbol_short!("ret"), symbol_short!("resm")), ());
    }

    pub fn paused(env: Env) -> bool {
        read_paused(&env)
    }

    pub fn set_verifier(env: Env, verifier: Address, authorized: bool) {
        let admin = read_admin(&env);
        admin.require_auth();
        let mut verifiers = read_verifiers(&env);
        if authorized {
            verifiers.set(verifier.clone(), true);
        } else {
            verifiers.remove(verifier.clone());
        }
        write_verifiers(&env, &verifiers);
        env.events().publish(
            (symbol_short!("ret"), symbol_short!("vrfy")),
            (verifier, authorized),
        );
    }

    pub fn is_verifier(env: Env, verifier: Address) -> bool {
        has_verifier(&env, &verifier)
    }

    pub fn retire(env: Env, caller: Address, token_ids: Vec<u64>, claim: ClaimData) -> BytesN<32> {
        caller.require_auth();

        if read_paused(&env) {
            panic_with_error!(&env, RetirementError::ContractPaused);
        }

        if token_ids.is_empty() {
            panic_with_error!(&env, RetirementError::NoTokensProvided);
        }

        if claim.period_start == 0 || claim.period_end == 0 || claim.period_end < claim.period_start
        {
            panic_with_error!(&env, RetirementError::InvalidClaimData);
        }

        if !has_rec_token(&env) {
            panic_with_error!(&env, RetirementError::RecTokenNotSet);
        }

        let rec_id = read_rec_token(&env);

        for i in 0..token_ids.len() {
            let token_id = token_ids.get(i).unwrap();

            for j in 0..i {
                if token_ids.get(j).unwrap() == token_id {
                    panic_with_error!(&env, RetirementError::DuplicateToken);
                }
            }

            if has_retired_token(&env, token_id) {
                panic_with_error!(&env, RetirementError::AlreadyRetired);
            }

            let owner: Address = env.invoke_contract(
                &rec_id,
                &Symbol::new(&env, "owner_of"),
                (token_id,).into_val(&env),
            );
            if owner != caller {
                panic_with_error!(&env, RetirementError::NotTokenOwner);
            }
        }

        let fee = read_retirement_fee(&env);
        if fee > 0 && has_usdc_token(&env) && has_fee_vault(&env) {
            let usdc_id = read_usdc_token(&env);
            let vault = read_fee_vault(&env);
            let _: () = env.invoke_contract(
                &usdc_id,
                &symbol_short!("xfer"),
                (caller.clone(), vault, fee).into_val(&env),
            );
        }

        for i in 0..token_ids.len() {
            let token_id = token_ids.get(i).unwrap();
            let _: () = env.invoke_contract(
                &rec_id,
                &Symbol::new(&env, "burn"),
                (caller.clone(), token_id).into_val(&env),
            );
        }

        let counter = read_receipt_counter(&env) + 1;
        write_receipt_counter(&env, counter);

        let receipt_id = compute_receipt_id(&env, counter);
        let timestamp = env.ledger().timestamp();
        let block_height = env.ledger().sequence() as u64;

        let receipt = RetirementReceipt {
            receipt_id: receipt_id.clone(),
            retirer: caller,
            token_ids: token_ids.clone(),
            total_mwh: token_ids.len() as u64,
            claim,
            timestamp,
            block_height,
        };

        write_receipt(&env, &receipt_id, &receipt);

        for i in 0..token_ids.len() {
            let token_id = token_ids.get(i).unwrap();
            write_retired_token(&env, token_id, &receipt_id);
        }

        env.events().publish(
            (symbol_short!("ret"), symbol_short!("retr")),
            (
                receipt_id.clone(),
                receipt.retirer.clone(),
                token_ids.len() as u64,
                receipt.total_mwh,
                receipt.claim.period_start,
                receipt.claim.period_end,
            ),
        );

        receipt_id
    }

    pub fn get_retirement_receipt(env: Env, receipt_id: BytesN<32>) -> RetirementReceipt {
        if !has_receipt(&env, &receipt_id) {
            panic_with_error!(&env, RetirementError::ReceiptNotFound);
        }
        read_receipt(&env, &receipt_id)
    }

    pub fn verify_retirement(env: Env, token_id: u64) -> Option<BytesN<32>> {
        read_retired_token(&env, token_id)
    }

    pub fn prove_claim(
        env: Env,
        wallet: Address,
        period_start: u64,
        period_end: u64,
    ) -> Vec<RetirementReceipt> {
        let counter = read_receipt_counter(&env);
        let mut results: Vec<RetirementReceipt> = Vec::new(&env);

        for i in 1..=counter {
            let rid = compute_receipt_id(&env, i);
            if has_receipt(&env, &rid) {
                let receipt = read_receipt(&env, &rid);
                if receipt.retirer == wallet
                    && receipt.claim.period_start >= period_start
                    && receipt.claim.period_end <= period_end
                {
                    results.push_back(receipt);
                }
            }
        }

        results
    }
}

fn compute_receipt_id(env: &Env, counter: u64) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    let counter_bytes = counter.to_be_bytes();
    bytes[24..32].copy_from_slice(&counter_bytes);
    BytesN::from_array(env, &bytes)
}
