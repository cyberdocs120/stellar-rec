use soroban_sdk::{
    testutils::Address as _, Address, Bytes, BytesN, Env,
};

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
