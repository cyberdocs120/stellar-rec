use soroban_sdk::{
    testutils::Address as _, Address, Bytes, BytesN, Env,
};

use crate::{OracleHandlerContract, OracleHandlerContractClient};

fn make_pubkey(env: &Env, val: u8) -> BytesN<32> {
    let mut arr = [0u8; 32];
    arr[0] = val;
    BytesN::from_array(env, &arr)
}

fn make_meter_id(env: &Env, val: u8) -> BytesN<32> {
    let mut arr = [0u8; 32];
    arr[0] = val;
    BytesN::from_array(env, &arr)
}

fn make_asset_id(env: &Env, val: u8) -> BytesN<32> {
    let mut arr = [0u8; 32];
    arr[0] = val;
    BytesN::from_array(env, &arr)
}

fn make_uri(env: &Env) -> Bytes {
    Bytes::from_slice(env, b"https://oracle.example.com")
}

#[test]
fn test_register_oracle() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    let pubkey = make_pubkey(&env, 1);
    let uri = make_uri(&env);
    client.register_oracle(&pubkey, &uri);

    assert_eq!(client.oracle_count(), 1);

    let node = client.get_oracle(&pubkey);
    assert_eq!(node.pubkey, pubkey);
    assert!(node.active);
}

#[test]
fn test_revoke_oracle() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    let pubkey = make_pubkey(&env, 1);
    let uri = make_uri(&env);
    client.register_oracle(&pubkey, &uri);
    client.revoke_oracle(&pubkey);

    let node = client.get_oracle(&pubkey);
    assert!(!node.active);
}

#[test]
fn test_set_threshold() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.set_threshold(&3u32, &5u32);
    let (n, d) = client.threshold();
    assert_eq!(n, 3);
    assert_eq!(d, 5);
}

#[test]
fn test_pause_resume() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    assert!(!client.paused());

    client.pause();
    assert!(client.paused());

    client.resume();
    assert!(!client.paused());
}

#[test]
fn test_set_meter() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    let meter_id = make_meter_id(&env, 1);
    let asset_id = make_asset_id(&env, 42);
    client.set_meter(&meter_id, &asset_id, &100u64);

    let binding = client.get_meter(&meter_id);
    assert_eq!(binding.asset_id, asset_id);
    assert_eq!(binding.capacity_mw, 100);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_duplicate_oracle_rejected() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    let pubkey = make_pubkey(&env, 1);
    let uri = make_uri(&env);
    client.register_oracle(&pubkey, &uri);
    client.register_oracle(&pubkey, &uri);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_get_nonexistent_oracle() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    let pubkey = make_pubkey(&env, 99);
    client.get_oracle(&pubkey);
}
