use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, Address, Bytes, BytesN, Env, vec, Vec,
};

use crate::{OracleHandlerContract, OracleHandlerContractClient};
use crate::types::FuelType;
use ed25519_dalek::{SigningKey, Signer};

#[contract]
pub struct MockRecToken;

#[contractimpl]
impl MockRecToken {
    pub fn mint(
        _env: Env,
        _to: Address,
        _asset_id: BytesN<32>,
        _generation_timestamp: u64,
        _fuel_type: FuelType,
        _vintage_year: u32,
        _metadata_uri: Bytes,
    ) -> u64 {
        123
    }
}

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
    let receiver = Address::generate(&env);
    let asset_id = make_asset_id(&env, 42);
    client.set_meter(&meter_id, &receiver, &asset_id, &100u64);

    let binding = client.get_meter(&meter_id);
    assert_eq!(binding.receiver, receiver);
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

#[test]
fn test_submit_reading_mints_token() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let rec_token_id = env.register(MockRecToken, ());
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.set_rec_token(&rec_token_id);

    // Setup Oracle
    use rand::thread_rng;

    let mut rng = thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let pubkey_bytes = verifying_key.to_bytes();
    let pubkey = BytesN::from_array(&env, &pubkey_bytes);

    client.register_oracle(&pubkey, &make_uri(&env));

    // Setup Meter
    let meter_id = make_meter_id(&env, 1);
    let receiver = Address::generate(&env);
    let asset_id = make_asset_id(&env, 42);
    client.set_meter(&meter_id, &receiver, &asset_id, &1000u64);

    // Prepare Reading
    let mwh = 500u64;
    let timestamp = 123456789u64;

    let mut data = Bytes::new(&env);
    data.append(&meter_id.clone().into());
    data.append(&Bytes::from_slice(&env, &mwh.to_be_bytes()));
    data.append(&Bytes::from_slice(&env, &timestamp.to_be_bytes()));

    let mut data_bytes = [0u8; 48];
    data.copy_into_slice(&mut data_bytes);
    let signature_bytes = signing_key.sign(&data_bytes).to_bytes();
    let signature = BytesN::from_array(&env, &signature_bytes);

    let signatures: Vec<(BytesN<32>, BytesN<64>)> = vec![&env, (pubkey, signature)];

    let token_id = client.submit_reading(
        &meter_id,
        &mwh,
        &timestamp,
        &signatures,
        &0u32, // Solar
        &2024u32,
        &make_uri(&env),
    );

    assert_eq!(token_id, 123);
}

#[test]
fn test_dispute_flow() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let rec_token_id = env.register(MockRecToken, ());
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);
    client.set_rec_token(&rec_token_id);

    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let pubkey = BytesN::from_array(&env, &signing_key.verifying_key().to_bytes());
    client.register_oracle(&pubkey, &make_uri(&env));

    let meter_id = make_meter_id(&env, 1);
    let receiver = Address::generate(&env);
    client.set_meter(&meter_id, &receiver, &make_asset_id(&env, 1), &1000u64);

    let mwh = 500u64;
    let timestamp = 123456789u64;
    let mut data = Bytes::new(&env);
    data.append(&meter_id.clone().into());
    data.append(&Bytes::from_slice(&env, &mwh.to_be_bytes()));
    data.append(&Bytes::from_slice(&env, &timestamp.to_be_bytes()));
    let reading_hash = env.crypto().sha256(&data);

    let mut data_bytes = [0u8; 48];
    data.copy_into_slice(&mut data_bytes);
    let sig = BytesN::from_array(&env, &signing_key.sign(&data_bytes).to_bytes());
    client.submit_reading(&meter_id, &mwh, &timestamp, &vec![&env, (pubkey, sig)], &0, &2024, &make_uri(&env));

    client.dispute(&reading_hash.clone().into());
    
    // Resolve
    client.resolve_dispute(&reading_hash.into(), &true);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_submit_while_paused() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.pause();
    client.submit_reading(&make_meter_id(&env, 1), &500, &123, &vec![&env], &0, &2024, &make_uri(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_submit_below_threshold_rejected() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.set_threshold(&2, &2);
    let meter_id = make_meter_id(&env, 1);
    let receiver = Address::generate(&env);
    client.set_meter(&meter_id, &receiver, &make_asset_id(&env, 1), &1000);
    
    // Only 1 signature
    let sig = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_reading(&meter_id, &500, &123, &vec![&env, (make_pubkey(&env, 1), sig)], &0, &2024, &make_uri(&env));
}
