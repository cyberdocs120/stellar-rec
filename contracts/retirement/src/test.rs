use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

use crate::{ClaimData, RetirementContract, RetirementContractClient};

mod mock_rec_token {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    #[contract]
    pub struct MockRecToken;

    #[contractimpl]
    impl MockRecToken {
        pub fn __constructor(_env: Env, _admin: Address) {}

        pub fn owner_of(env: Env, _token_id: u64) -> Address {
            env.storage()
                .instance()
                .get(&soroban_sdk::symbol_short!("owner"))
                .unwrap()
        }

        pub fn set_owner(env: Env, owner: Address) {
            env.storage()
                .instance()
                .set(&soroban_sdk::symbol_short!("owner"), &owner);
        }

        pub fn burn(_env: Env, _caller: Address, _token_id: u64) {}
    }
}

mod mock_usdc {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    #[contract]
    pub struct MockUsdc;

    #[contractimpl]
    impl MockUsdc {
        pub fn __constructor(_env: Env, _admin: Address) {}

        pub fn xfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
    }
}

fn make_claim_data(env: &Env) -> ClaimData {
    ClaimData {
        period_start: 1700000000,
        period_end: 1702592000,
        purpose: Bytes::from_slice(env, b"Scope 2 market-based GHG accounting"),
        jurisdiction: Bytes::from_slice(env, b"US / RE100"),
    }
}

#[test]
fn test_retire_single_token() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let retirer = Address::generate(&env);

    let rec_id = env.register(mock_rec_token::MockRecToken, (&admin,));
    let mock_rec = mock_rec_token::MockRecTokenClient::new(&env, &rec_id);
    mock_rec.set_owner(&retirer);

    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_id);

    let token_ids = Vec::from_array(&env, [1u64]);
    let claim = make_claim_data(&env);

    let receipt_id = client.retire(&retirer, &token_ids, &claim);
    let receipt = client.get_retirement_receipt(&receipt_id);

    assert_eq!(receipt.retirer, retirer);
    assert_eq!(receipt.total_mwh, 1);
    assert_eq!(receipt.token_ids.len(), 1);
    assert_eq!(receipt.token_ids.get(0).unwrap(), 1);
}

#[test]
fn test_retire_multiple_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let retirer = Address::generate(&env);

    let rec_id = env.register(mock_rec_token::MockRecToken, (&admin,));
    let mock_rec = mock_rec_token::MockRecTokenClient::new(&env, &rec_id);
    mock_rec.set_owner(&retirer);

    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_id);

    let token_ids = Vec::from_array(&env, [1u64, 2u64, 3u64]);
    let claim = make_claim_data(&env);

    let receipt_id = client.retire(&retirer, &token_ids, &claim);
    let receipt = client.get_retirement_receipt(&receipt_id);

    assert_eq!(receipt.total_mwh, 3);
    assert_eq!(receipt.token_ids.len(), 3);
}

#[test]
fn test_verify_retirement() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let retirer = Address::generate(&env);

    let rec_id = env.register(mock_rec_token::MockRecToken, (&admin,));
    let mock_rec = mock_rec_token::MockRecTokenClient::new(&env, &rec_id);
    mock_rec.set_owner(&retirer);

    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_id);

    let token_ids = Vec::from_array(&env, [1u64]);
    let claim = make_claim_data(&env);
    let receipt_id = client.retire(&retirer, &token_ids, &claim);

    let verified = client.verify_retirement(&1u64);
    assert_eq!(verified, Some(receipt_id));

    let not_retired = client.verify_retirement(&999u64);
    assert_eq!(not_retired, None);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_retire_empty_token_list() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let retirer = Address::generate(&env);

    let rec_id = env.register(mock_rec_token::MockRecToken, (&admin,));
    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_id);

    let token_ids = Vec::from_array(&env, []);
    let claim = make_claim_data(&env);
    client.retire(&retirer, &token_ids, &claim);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_retire_invalid_claim_period() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let retirer = Address::generate(&env);

    let rec_id = env.register(mock_rec_token::MockRecToken, (&admin,));
    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_id);

    let token_ids = Vec::from_array(&env, [1u64]);
    let claim = ClaimData {
        period_start: 100,
        period_end: 50,
        purpose: Bytes::new(&env),
        jurisdiction: Bytes::new(&env),
    };
    client.retire(&retirer, &token_ids, &claim);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_get_nonexistent_receipt() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);

    let fake_id = BytesN::from_array(&env, &[0u8; 32]);
    client.get_retirement_receipt(&fake_id);
}

#[test]
fn test_prove_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let retirer = Address::generate(&env);
    let other = Address::generate(&env);

    let rec_id = env.register(mock_rec_token::MockRecToken, (&admin,));
    let mock_rec = mock_rec_token::MockRecTokenClient::new(&env, &rec_id);
    mock_rec.set_owner(&retirer);

    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_id);

    let token_ids = Vec::from_array(&env, [1u64, 2u64]);
    let claim = make_claim_data(&env);
    client.retire(&retirer, &token_ids, &claim);

    let results = client.prove_claim(&retirer, &0u64, &9999999999u64);
    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().retirer, retirer);

    let other_results = client.prove_claim(&other, &0u64, &9999999999u64);
    assert_eq!(other_results.len(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_retire_duplicate_token_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let retirer = Address::generate(&env);

    let rec_id = env.register(mock_rec_token::MockRecToken, (&admin,));
    let mock_rec = mock_rec_token::MockRecTokenClient::new(&env, &rec_id);
    mock_rec.set_owner(&retirer);

    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_id);
    client.set_retirement_fee(&0i128);

    let token_ids = Vec::from_array(&env, [1u64, 1u64]);
    client.retire(&retirer, &token_ids, &make_claim_data(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_retire_while_paused_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let retirer = Address::generate(&env);

    let rec_id = env.register(mock_rec_token::MockRecToken, (&admin,));
    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_id);
    client.set_retirement_fee(&0i128);
    client.pause();

    let token_ids = Vec::from_array(&env, [1u64]);
    client.retire(&retirer, &token_ids, &make_claim_data(&env));
}

#[test]
fn test_pause_resume() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);

    assert!(!client.paused());
    client.pause();
    assert!(client.paused());
    client.resume();
    assert!(!client.paused());
}

#[test]
fn test_admin_functions() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let contract_id = env.register(RetirementContract, (&admin,));
    let client = RetirementContractClient::new(&env, &contract_id);

    assert_eq!(client.admin(), admin);

    client.transfer_admin(&new_admin);
    assert_eq!(client.admin(), new_admin);

    let fee = 5_000_000i128;
    client.set_retirement_fee(&fee);
    assert_eq!(client.retirement_fee(), fee);
}
