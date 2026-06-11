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

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
    pub fn xfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
    pub fn balance(_env: Env, _id: Address) -> i128 {
        1_000_000_000_000
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

fn setup_env(env: &Env) -> (Address, OracleHandlerContractClient<'_>) {
    let admin = Address::generate(env);
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(env, &contract_id);
    (admin, client)
}

#[test]
fn test_register_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    let operator = Address::generate(&env);
    let pubkey = make_pubkey(&env, 1);
    let uri = make_uri(&env);
    client.register_oracle(&pubkey, &uri, &operator);

    assert_eq!(client.oracle_count(), 1);

    let node = client.get_oracle(&pubkey);
    assert_eq!(node.pubkey, pubkey);
    assert_eq!(node.operator, operator);
    assert_eq!(node.stake, 0);
    assert_eq!(node.rewards, 0);
    assert_eq!(node.reputation, 0);
    assert!(!node.active);
}

#[test]
fn test_revoke_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    let operator = Address::generate(&env);
    let pubkey = make_pubkey(&env, 1);
    let uri = make_uri(&env);
    client.register_oracle(&pubkey, &uri, &operator);
    client.revoke_oracle(&pubkey);

    let node = client.get_oracle(&pubkey);
    assert!(!node.active);
}

#[test]
fn test_set_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    client.set_threshold(&3u32, &5u32);
    let (n, d) = client.threshold();
    assert_eq!(n, 3);
    assert_eq!(d, 5);
}

#[test]
fn test_pause_resume() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    assert!(!client.paused());
    client.pause();
    assert!(client.paused());
    client.resume();
    assert!(!client.paused());
}

#[test]
fn test_set_meter() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
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
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    let operator = Address::generate(&env);
    let pubkey = make_pubkey(&env, 1);
    let uri = make_uri(&env);
    client.register_oracle(&pubkey, &uri, &operator);
    client.register_oracle(&pubkey, &uri, &operator);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_get_nonexistent_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    let pubkey = make_pubkey(&env, 99);
    client.get_oracle(&pubkey);
}

#[test]
fn test_deposit_bond_activates_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);

    let operator = Address::generate(&env);
    let usdc_token = env.register(MockToken, ());
    client.set_usdc_token(&usdc_token);

    let pubkey = make_pubkey(&env, 1);
    client.register_oracle(&pubkey, &make_uri(&env), &operator);

    // Oracle starts inactive
    let node = client.get_oracle(&pubkey);
    assert!(!node.active);

    // Deposit bond — meets minimum bond of 100 yUSDC
    client.deposit_bond(&pubkey, &100_000_000_000i128);

    let node = client.get_oracle(&pubkey);
    assert_eq!(node.stake, 100_000_000_000);
    assert!(node.active);
}

#[test]
fn test_withdraw_bond_deactivates_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);

    let operator = Address::generate(&env);
    let usdc_token = env.register(MockToken, ());
    client.set_usdc_token(&usdc_token);

    let pubkey = make_pubkey(&env, 1);
    client.register_oracle(&pubkey, &make_uri(&env), &operator);

    // Deposit bond
    client.deposit_bond(&pubkey, &100_000_000_000i128);
    assert!(client.get_oracle(&pubkey).active);

    // Withdraw — drops below minimum bond
    client.withdraw_bond(&pubkey, &100_000_000_000i128);

    let node = client.get_oracle(&pubkey);
    assert_eq!(node.stake, 0);
    assert!(!node.active);
}

#[test]
fn test_submit_reading_with_bond() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = env.register(MockToken, ());
    let rec_token_id = env.register(MockRecToken, ());
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.set_rec_token(&rec_token_id);
    client.set_usdc_token(&usdc_token);

    // Setup Oracle with bond
    use rand::thread_rng;
    let mut rng = thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let pubkey_bytes = verifying_key.to_bytes();
    let pubkey = BytesN::from_array(&env, &pubkey_bytes);

    let operator = Address::generate(&env);
    client.register_oracle(&pubkey, &make_uri(&env), &operator);
    client.deposit_bond(&pubkey, &100_000_000_000i128);

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
#[should_panic(expected = "Error(Contract, #11)")]
fn test_submit_reading_with_low_bond_rejected() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let rec_token_id = env.register(MockRecToken, ());
    let usdc_token = env.register(MockToken, ());
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.set_rec_token(&rec_token_id);
    client.set_usdc_token(&usdc_token);

    // Setup Oracle with bond
    use rand::thread_rng;
    let mut rng = thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let pubkey_bytes = verifying_key.to_bytes();
    let pubkey = BytesN::from_array(&env, &pubkey_bytes);

    let operator = Address::generate(&env);
    client.register_oracle(&pubkey, &make_uri(&env), &operator);
    client.deposit_bond(&pubkey, &100_000_000_000i128);

    // Raise minimum bond above the oracle's stake
    client.set_minimum_bond(&200_000_000_000i128);

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

    client.submit_reading(
        &meter_id,
        &mwh,
        &timestamp,
        &signatures,
        &0u32,
        &2024u32,
        &make_uri(&env),
    );
}

#[test]
fn test_dispute_and_slashing() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = env.register(MockToken, ());
    let rec_token_id = env.register(MockRecToken, ());
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.set_rec_token(&rec_token_id);
    client.set_usdc_token(&usdc_token);

    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let pubkey = BytesN::from_array(&env, &signing_key.verifying_key().to_bytes());

    let operator = Address::generate(&env);
    client.register_oracle(&pubkey, &make_uri(&env), &operator);
    client.deposit_bond(&pubkey, &100_000_000_000i128);

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
    client.submit_reading(&meter_id, &mwh, &timestamp, &vec![&env, (pubkey.clone(), sig)], &0, &2024, &make_uri(&env));

    client.dispute(&reading_hash.clone().into());

    // Resolve with outcome=false (fraudulent) — should slash
    client.resolve_dispute(&reading_hash.into(), &false);

    let node = client.get_oracle(&pubkey);
    assert_eq!(node.stake, 0, "Stake should be slashed to 0");
    assert!(!node.active, "Oracle should be deactivated after slashing");
    assert_eq!(node.reputation, 0u32.saturating_sub(10), "Reputation should be reduced"); // 0 - 10 = 0 (saturating)
}

#[test]
fn test_dispute_honest_outcome_boosts_reputation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = env.register(MockToken, ());
    let rec_token_id = env.register(MockRecToken, ());
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.set_rec_token(&rec_token_id);
    client.set_usdc_token(&usdc_token);

    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let pubkey = BytesN::from_array(&env, &signing_key.verifying_key().to_bytes());

    let operator = Address::generate(&env);
    client.register_oracle(&pubkey, &make_uri(&env), &operator);
    client.deposit_bond(&pubkey, &100_000_000_000i128);

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
    client.submit_reading(&meter_id, &mwh, &timestamp, &vec![&env, (pubkey.clone(), sig)], &0, &2024, &make_uri(&env));

    client.dispute(&reading_hash.clone().into());

    // Resolve with outcome=true (honest) — should boost reputation
    client.resolve_dispute(&reading_hash.into(), &true);

    let node = client.get_oracle(&pubkey);
    assert_eq!(node.stake, 100_000_000_000, "Stake should remain intact");
    assert_eq!(node.reputation, 5, "Reputation should be boosted by 5");
}



#[test]
fn test_claim_rewards_direct() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = env.register(MockToken, ());
    let contract_id = env.register(OracleHandlerContract, (&admin,));
    let client = OracleHandlerContractClient::new(&env, &contract_id);

    client.set_usdc_token(&usdc_token);

    let operator = Address::generate(&env);
    let pubkey = make_pubkey(&env, 1);
    client.register_oracle(&pubkey, &make_uri(&env), &operator);

    // Manually set rewards by using a real oracle reading with valid signatures
    let rec_token_id = env.register(MockRecToken, ());
    client.set_rec_token(&rec_token_id);

    // Setup oracle with bond
    client.deposit_bond(&pubkey, &100_000_000_000i128);

    // Fund reward pool
    client.fund_reward_pool(&10_000_000_000i128);

    let meter_id = make_meter_id(&env, 1);
    let receiver = Address::generate(&env);
    client.set_meter(&meter_id, &receiver, &make_asset_id(&env, 42), &1000u64);

    // Submit reading with valid signature
    use rand::thread_rng;
    let mut rng = thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let real_pubkey_bytes = verifying_key.to_bytes();
    let real_pubkey = BytesN::from_array(&env, &real_pubkey_bytes);

    // Re-register the oracle with the real pubkey
    // We need to use the real pubkey for proper ed25519 verification
    // Let's use the signing key directly
    let operator2 = Address::generate(&env);
    client.register_oracle(&real_pubkey, &make_uri(&env), &operator2);
    client.deposit_bond(&real_pubkey, &100_000_000_000i128);

    let mwh = 500u64;
    let timestamp = 123456789u64;
    let mut data = Bytes::new(&env);
    data.append(&meter_id.clone().into());
    data.append(&Bytes::from_slice(&env, &mwh.to_be_bytes()));
    data.append(&Bytes::from_slice(&env, &timestamp.to_be_bytes()));

    let mut data_bytes = [0u8; 48];
    data.copy_into_slice(&mut data_bytes);
    let real_sig_bytes = signing_key.sign(&data_bytes).to_bytes();
    let real_sig = BytesN::from_array(&env, &real_sig_bytes);

    let signatures: Vec<(BytesN<32>, BytesN<64>)> = vec![&env, (real_pubkey.clone(), real_sig)];

    client.submit_reading(
        &meter_id,
        &mwh,
        &timestamp,
        &signatures,
        &0u32,
        &2024u32,
        &make_uri(&env),
    );

    let node = client.get_oracle(&real_pubkey);
    assert!(node.rewards > 0, "Oracle should have accumulated rewards");

    // Claim rewards
    client.claim_rewards(&real_pubkey);

    let node_after = client.get_oracle(&real_pubkey);
    assert_eq!(node_after.rewards, 0, "Rewards should be zero after claim");
}

#[test]
fn test_set_usdc_token_and_minimum_bond() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);

    let usdc_token = Address::generate(&env);
    client.set_usdc_token(&usdc_token);
    assert_eq!(client.usdc_token(), usdc_token);

    assert_eq!(client.minimum_bond(), 100_000_000_000);

    client.set_minimum_bond(&200_000_000_000i128);
    assert_eq!(client.minimum_bond(), 200_000_000_000);
}

#[test]
fn test_set_reward_per_reading() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    assert_eq!(client.reward_per_reading(), 1_000_000);

    client.set_reward_per_reading(&2_000_000i128);
    assert_eq!(client.reward_per_reading(), 2_000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_submit_while_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    client.pause();
    client.submit_reading(&make_meter_id(&env, 1), &500, &123, &vec![&env], &0, &2024, &make_uri(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_submit_below_threshold_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup_env(&env);
    client.set_threshold(&2, &2);
    let meter_id = make_meter_id(&env, 1);
    let receiver = Address::generate(&env);
    client.set_meter(&meter_id, &receiver, &make_asset_id(&env, 1), &1000);

    // Only 1 signature
    let sig = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_reading(&meter_id, &500, &123, &vec![&env, (make_pubkey(&env, 1), sig)], &0, &2024, &make_uri(&env));
}
