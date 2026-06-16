use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

use crate::{FuelType, RecTokenContract, RecTokenContractClient};

macro_rules! setup_env {
    ($env:ident, $client:ident, $admin:ident, $minter:ident, $receiver:ident) => {
        let $env = Env::default();
        let $admin = Address::generate(&$env);
        let $minter = Address::generate(&$env);
        let $receiver = Address::generate(&$env);
        $env.mock_all_auths();
        let contract_id = $env.register(RecTokenContract, (&$admin,));
        let $client = RecTokenContractClient::new(&$env, &contract_id);
        $client.authorize_minter(&$minter);
    };
}

fn make_asset_id(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0; 32])
}

fn make_metadata_uri(env: &Env) -> Bytes {
    Bytes::new(env)
}

#[test]
fn test_mint_increases_supply() {
    setup_env!(env, client, _admin, _minter, receiver);

    env.mock_all_auths();
    let token_id = client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );

    assert_eq!(client.total_supply(), 1);
    assert_eq!(client.balance_of(&receiver), 1);
    assert_eq!(client.owner_of(&token_id), receiver);
}

#[test]
fn test_mint_multiple_tokens() {
    setup_env!(env, client, _admin, _minter, _receiver);
    let receiver1 = Address::generate(&env);
    let receiver2 = Address::generate(&env);

    env.mock_all_auths();
    let id1 = client.mint(
        &receiver1,
        &make_asset_id(&env),
        &100u64,
        &FuelType::Wind,
        &2025u32,
        &make_metadata_uri(&env),
    );
    let id2 = client.mint(
        &receiver2,
        &make_asset_id(&env),
        &200u64,
        &FuelType::Hydro,
        &2026u32,
        &make_metadata_uri(&env),
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(client.total_supply(), 2);
    assert_eq!(client.balance_of(&receiver1), 1);
    assert_eq!(client.balance_of(&receiver2), 1);
}

#[test]
fn test_transfer_changes_owner() {
    setup_env!(env, client, _admin, _minter, receiver);
    let new_owner = Address::generate(&env);

    env.mock_all_auths();
    let token_id = client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );

    client.transfer(&receiver, &new_owner, &token_id);

    assert_eq!(client.owner_of(&token_id), new_owner);
    assert_eq!(client.balance_of(&receiver), 0);
    assert_eq!(client.balance_of(&new_owner), 1);
}

#[test]
fn test_burn_retires_token() {
    setup_env!(env, client, _admin, _minter, receiver);

    env.mock_all_auths();
    let token_id = client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );

    let caller = receiver.clone();
    client.burn(&caller, &token_id);

    let meta = client.token_metadata(&token_id);
    assert_eq!(meta.state, crate::RecState::Retired);
    assert!(meta.retired_at.is_some());
    assert_eq!(client.total_supply(), 0);
    assert_eq!(client.balance_of(&receiver), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_double_burn_rejected() {
    setup_env!(env, client, _admin, _minter, receiver);

    env.mock_all_auths();
    let token_id = client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );

    let caller = receiver.clone();
    client.burn(&caller, &token_id);
    client.burn(&caller, &token_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_non_owner_transfer_rejected() {
    setup_env!(env, client, _admin, _minter, receiver);
    let other = Address::generate(&env);

    env.mock_all_auths();
    let token_id = client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );

    client.transfer(&other, &receiver, &token_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_burn_nonexistent_token() {
    setup_env!(env, client, _admin, _minter, _receiver);

    let caller = Address::generate(&env);
    env.mock_all_auths();
    client.burn(&caller, &999u64);
}

#[test]
fn test_set_metadata_uri() {
    setup_env!(env, client, _admin, minter, receiver);

    env.mock_all_auths();
    let token_id = client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );

    let new_uri = Bytes::from_slice(&env, b"https://example.com/new-metadata");
    client.set_metadata_uri(&token_id, &new_uri);

    let meta = client.token_metadata(&token_id);
    assert_eq!(meta.metadata_uri, new_uri);
}

#[test]
fn test_tokens_by_owner_pagination() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let owner = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(RecTokenContract, (&admin,));
    let client = RecTokenContractClient::new(&env, &contract_id);
    client.authorize_minter(&minter);

    for i in 0..10 {
        client.mint(
            &owner,
            &make_asset_id(&env),
            &(1000 + i),
            &FuelType::Wind,
            &2026u32,
            &make_metadata_uri(&env),
        );
    }

    let page1 = client.tokens_by_owner(&owner, &0u64, &3u64);
    assert_eq!(page1.len(), 3);
    assert_eq!(page1.get(0).unwrap(), 1);
    assert_eq!(page1.get(1).unwrap(), 2);
    assert_eq!(page1.get(2).unwrap(), 3);

    let page2 = client.tokens_by_owner(&owner, &3u64, &3u64);
    assert_eq!(page2.len(), 3);
    assert_eq!(page2.get(0).unwrap(), 4);
    assert_eq!(page2.get(1).unwrap(), 5);
    assert_eq!(page2.get(2).unwrap(), 6);

    let page_all = client.tokens_by_owner(&owner, &0u64, &20u64);
    assert_eq!(page_all.len(), 10);

    let other = Address::generate(&env);
    let empty = client.tokens_by_owner(&other, &0u64, &10u64);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_pause_resume() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(RecTokenContract, (&admin,));
    let client = RecTokenContractClient::new(&env, &contract_id);

    assert!(!client.paused());
    client.pause();
    assert!(client.paused());
    client.resume();
    assert!(!client.paused());
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_mint_while_paused_rejected() {
    setup_env!(env, client, _admin, minter, receiver);

    env.mock_all_auths();
    client.pause();
    client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_transfer_while_paused_rejected() {
    setup_env!(env, client, _admin, _minter, receiver);
    let new_owner = Address::generate(&env);

    env.mock_all_auths();
    let token_id = client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );

    client.pause();
    client.transfer(&receiver, &new_owner, &token_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_transfer_retired_token_rejected() {
    setup_env!(env, client, _admin, _minter, receiver);

    env.mock_all_auths();
    let token_id = client.mint(
        &receiver,
        &make_asset_id(&env),
        &1234567890u64,
        &FuelType::Solar,
        &2026u32,
        &make_metadata_uri(&env),
    );

    client.burn(&receiver, &token_id);

    let new_owner = Address::generate(&env);
    client.transfer(&receiver, &new_owner, &token_id);
}

#[test]
fn test_admin_functions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(RecTokenContract, (&admin,));
    let client = RecTokenContractClient::new(&env, &contract_id);

    assert_eq!(client.admin(), admin);
    client.transfer_admin(&new_admin);
    assert_eq!(client.admin(), new_admin);
}
