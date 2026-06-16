#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use marketplace::{
        MarketplaceContract, MarketplaceContractClient, OrderRestriction, OrderSide, PositionState,
    };
    use oracle_handler::{OracleHandlerContract, OracleHandlerContractClient};
    use rec_token::{FuelType, RecTokenContract, RecTokenContractClient};
    use retirement::{ClaimData, RetirementContract, RetirementContractClient};
    use soroban_sdk::{
        contract, contractimpl,
        testutils::{Address as _, Ledger as _},
        vec, Address, Bytes, BytesN, Env, Vec,
    };

    #[contract]
    pub struct MockUsdc;

    #[contractimpl]
    impl MockUsdc {
        pub fn __constructor(_env: Env, _admin: Address) {}

        pub fn xfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
    }

    struct Protocol<'a> {
        env: &'a Env,
        admin: Address,
        oracle_operator: Address,
        generator: Address,
        buyer: Address,
        retiree: Address,
        bot: Address,
        vault: Address,
        rec_client: RecTokenContractClient<'a>,
        oracle_client: OracleHandlerContractClient<'a>,
        market_client: MarketplaceContractClient<'a>,
        retirement_client: RetirementContractClient<'a>,
    }

    fn make_bytes_n(env: &Env, seed: u8) -> BytesN<32> {
        BytesN::from_array(env, &[seed; 32])
    }

    fn make_uri(env: &Env) -> Bytes {
        Bytes::from_slice(env, b"ipfs://stellar-rec")
    }

    fn make_claim(env: &Env) -> ClaimData {
        ClaimData {
            period_start: 1_704_067_200,
            period_end: 1_711_929_600,
            purpose: Bytes::from_slice(env, b"Scope 2 market-based GHG accounting"),
            jurisdiction: Bytes::from_slice(env, b"US / RE100"),
        }
    }

    fn oracle_key(env: &Env, seed: u8) -> (SigningKey, BytesN<32>) {
        let signing_key = SigningKey::from_bytes(&[seed; 32]);
        let pubkey = BytesN::from_array(env, &signing_key.verifying_key().to_bytes());
        (signing_key, pubkey)
    }

    fn signed_reading(
        env: &Env,
        signing_key: &SigningKey,
        meter_id: &BytesN<32>,
        mwh: u64,
        timestamp: u64,
    ) -> (BytesN<32>, BytesN<64>, BytesN<32>) {
        let mut data = Bytes::new(env);
        data.append(&meter_id.clone().into());
        data.append(&Bytes::from_slice(env, &mwh.to_be_bytes()));
        data.append(&Bytes::from_slice(env, &timestamp.to_be_bytes()));

        let mut data_bytes = [0u8; 48];
        data.copy_into_slice(&mut data_bytes);
        let signature = BytesN::from_array(env, &signing_key.sign(&data_bytes).to_bytes());
        let reading_hash: BytesN<32> = env.crypto().sha256(&data).into();

        (reading_hash, signature, meter_id.clone())
    }

    fn setup_protocol(env: &Env) -> Protocol<'_> {
        env.mock_all_auths_allowing_non_root_auth();

        let admin = Address::generate(env);
        let oracle_operator = Address::generate(env);
        let generator = Address::generate(env);
        let buyer = Address::generate(env);
        let retiree = Address::generate(env);
        let bot = Address::generate(env);
        let vault = Address::generate(env);

        let rec_id = env.register(RecTokenContract, (&admin,));
        let oracle_id = env.register(OracleHandlerContract, (&admin,));
        let market_id = env.register(MarketplaceContract, (&admin,));
        let retirement_id = env.register(RetirementContract, (&admin,));
        let usdc_id = env.register(MockUsdc, (&admin,));

        let rec_client = RecTokenContractClient::new(env, &rec_id);
        let oracle_client = OracleHandlerContractClient::new(env, &oracle_id);
        let market_client = MarketplaceContractClient::new(env, &market_id);
        let retirement_client = RetirementContractClient::new(env, &retirement_id);

        rec_client.authorize_minter(&oracle_id);

        oracle_client.set_rec_token(&rec_id);
        oracle_client.set_usdc_token(&usdc_id);
        oracle_client.set_minimum_bond(&100_000_000_000i128);

        market_client.set_rec_token(&rec_id);
        market_client.set_usdc_token(&usdc_id);
        market_client.set_fee_vault(&vault);
        market_client.set_oracle_handler(&oracle_id);

        retirement_client.set_rec_token(&rec_id);
        retirement_client.set_usdc_token(&usdc_id);
        retirement_client.set_fee_vault(&vault);
        retirement_client.set_retirement_fee(&0i128);

        let (_signing_key, pubkey) = oracle_key(env, 7);
        oracle_client.register_oracle(&pubkey, &make_uri(env), &oracle_operator);
        oracle_client.deposit_bond(&pubkey, &100_000_000_000i128);

        let meter_id = make_bytes_n(env, 11);
        let asset_id = make_bytes_n(env, 23);
        oracle_client.set_meter(&meter_id, &generator, &asset_id, &1_000u64);
        oracle_client.set_price(&5_000_000i128);

        Protocol {
            env,
            admin,
            oracle_operator,
            generator,
            buyer,
            retiree,
            bot,
            vault,
            rec_client,
            oracle_client,
            market_client,
            retirement_client,
        }
    }

    fn setup_marketplace(env: &Env) -> MarketplaceContractClient<'_> {
        env.mock_all_auths_allowing_non_root_auth();

        let admin = Address::generate(env);
        let vault = Address::generate(env);
        let rec_id = env.register(RecTokenContract, (&admin,));
        let market_id = env.register(MarketplaceContract, (&admin,));
        let usdc_id = env.register(MockUsdc, (&admin,));

        let market_client = MarketplaceContractClient::new(env, &market_id);
        market_client.set_rec_token(&rec_id);
        market_client.set_usdc_token(&usdc_id);
        market_client.set_fee_vault(&vault);
        market_client
    }

    #[test]
    fn test_full_rec_lifecycle_mint_trade_retire_verify() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let meter_id = make_bytes_n(&env, 11);
        let (signing_key, pubkey) = oracle_key(&env, 7);
        let (_reading_hash, signature, _) =
            signed_reading(&env, &signing_key, &meter_id, 1, 1_704_067_200);
        let signatures: Vec<(BytesN<32>, BytesN<64>)> = vec![&env, (pubkey, signature)];

        let token_id = protocol.oracle_client.submit_reading(
            &meter_id,
            &1u64,
            &1_704_067_200u64,
            &signatures,
            &0u32,
            &2026u32,
            &make_uri(&env),
        );

        assert_eq!(protocol.rec_client.owner_of(&token_id), protocol.generator);

        let price = 5_000_000i128;
        protocol.market_client.place_order(
            &protocol.generator,
            &OrderSide::Sell,
            &price,
            &1u64,
            &OrderRestriction::None,
            &Some(2026u32),
        );
        protocol.market_client.place_order(
            &protocol.buyer,
            &OrderSide::Buy,
            &price,
            &1u64,
            &OrderRestriction::None,
            &Some(2026u32),
        );
        let (fill_qty, fill_price, fee) = protocol.market_client.match_orders(&2u64, &1u64);
        assert_eq!(fill_qty, 1);
        assert_eq!(fill_price, price);
        assert_eq!(fee, 5000);
        assert_eq!(protocol.rec_client.owner_of(&token_id), protocol.buyer);

        let token_ids = Vec::from_array(&env, [token_id]);
        let receipt_id =
            protocol
                .retirement_client
                .retire(&protocol.buyer, &token_ids, &make_claim(&env));
        let receipt = protocol
            .retirement_client
            .get_retirement_receipt(&receipt_id);

        assert_eq!(receipt.retirer, protocol.buyer);
        assert_eq!(receipt.total_mwh, 1);
        assert_eq!(receipt.token_ids.len(), 1);
        assert_eq!(
            receipt.claim.jurisdiction,
            Bytes::from_slice(&env, b"US / RE100")
        );
        assert_eq!(
            protocol.retirement_client.verify_retirement(&token_id),
            Some(receipt_id)
        );
        assert_eq!(protocol.rec_client.balance_of(&protocol.buyer), 0);
    }

    #[test]
    fn test_cfd_liquidation_success_and_rejection_when_collateralized() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let producer = Address::generate(&env);
        let offtaker = Address::generate(&env);
        let strike = 40_000_000i128;
        let qty = 5_000u64;
        let expiry = env.ledger().timestamp() + 86_400;
        let collateral = (qty as i128) * strike * 1_500 / 10_000;

        let pos_id =
            protocol
                .market_client
                .open_cfd(&producer, &strike, &qty, &expiry, &collateral);
        protocol
            .market_client
            .accept_cfd(&offtaker, &pos_id, &collateral);

        protocol.oracle_client.set_price(&40_000_000i128);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            protocol.market_client.liquidate(&protocol.bot, &pos_id);
        }));
        assert!(result.is_err());

        protocol.oracle_client.set_price(&50_000_000i128);
        protocol.market_client.liquidate(&protocol.bot, &pos_id);

        let position = protocol.market_client.get_cfd(&pos_id);
        assert_eq!(position.state, PositionState::Liquidated);
        assert_eq!(position.mtm_value, 50_000_000_000i128);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn test_vintage_mismatch_rejected() {
        let env = Env::default();
        let market_client = setup_marketplace(&env);
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);

        market_client.place_order(
            &buyer,
            &OrderSide::Buy,
            &5_000_000i128,
            &1u64,
            &OrderRestriction::None,
            &Some(2027u32),
        );
        market_client.place_order(
            &seller,
            &OrderSide::Sell,
            &5_000_000i128,
            &1u64,
            &OrderRestriction::None,
            &Some(2026u32),
        );
        market_client.match_orders(&1u64, &2u64);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #10)")]
    fn test_oracle_pause_blocks_minting() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let meter_id = make_bytes_n(&env, 11);
        let (signing_key, pubkey) = oracle_key(&env, 7);
        let (_, signature, _) = signed_reading(&env, &signing_key, &meter_id, 1, 1_704_067_200);
        let signatures: Vec<(BytesN<32>, BytesN<64>)> = vec![&env, (pubkey, signature)];

        protocol.oracle_client.pause();
        protocol.oracle_client.submit_reading(
            &meter_id,
            &1u64,
            &1_704_067_200u64,
            &signatures,
            &0u32,
            &2026u32,
            &make_uri(&env),
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #14)")]
    fn test_marketplace_pause_blocks_trading() {
        let env = Env::default();
        let market_client = setup_marketplace(&env);
        let trader = Address::generate(&env);

        market_client.pause();
        market_client.place_order(
            &trader,
            &OrderSide::Buy,
            &5_000_000i128,
            &1u64,
            &OrderRestriction::None,
            &None,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #10)")]
    fn test_retirement_pause_blocks_retiring() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let retirer = Address::generate(&env);
        let contract_id = env.register(RetirementContract, (&admin,));
        let retirement_client = RetirementContractClient::new(&env, &contract_id);
        let token_ids = Vec::from_array(&env, [1u64]);

        retirement_client.pause();
        retirement_client.retire(&retirer, &token_ids, &make_claim(&env));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #6)")]
    fn test_rec_token_pause_blocks_minting() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let minter = Address::generate(&env);
        let receiver = Address::generate(&env);
        let contract_id = env.register(RecTokenContract, (&admin,));
        let rec_client = RecTokenContractClient::new(&env, &contract_id);

        rec_client.authorize_minter(&minter);
        rec_client.pause();
        rec_client.mint(
            &receiver,
            &make_bytes_n(&env, 1),
            &1_704_067_200u64,
            &FuelType::Solar,
            &2026u32,
            &make_uri(&env),
        );
    }

    #[test]
    fn test_cfd_settlement_full_lifecycle() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let producer = Address::generate(&env);
        let offtaker = Address::generate(&env);
        let strike = 40_000_000i128;
        let qty = 5_000u64;
        let now = env.ledger().timestamp();
        let expiry = now + 1;
        let collateral = (qty as i128) * strike * 1_500 / 10_000;

        let pos_id =
            protocol
                .market_client
                .open_cfd(&producer, &strike, &qty, &expiry, &collateral);
        protocol
            .market_client
            .accept_cfd(&offtaker, &pos_id, &collateral);

        let mut position = protocol.market_client.get_cfd(&pos_id);
        assert_eq!(position.strike_price, strike);
        assert_eq!(position.quantity, qty);
        assert_eq!(position.counterparty_a, producer);
        assert_eq!(position.counterparty_b, Some(offtaker));
        assert_eq!(position.state, PositionState::Active);

        protocol.env.ledger().set_timestamp(now + 2);
        protocol.oracle_client.set_price(&30_000_000i128);
        protocol.market_client.settle_cfd(&producer, &pos_id);

        position = protocol.market_client.get_cfd(&pos_id);
        assert_eq!(position.state, PositionState::Settled);
        assert_eq!(position.mtm_value, -50_000_000_000i128);
    }

    #[test]
    fn test_retire_multiple_tokens_with_fee() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let meter_id = make_bytes_n(&env, 11);
        let (signing_key, pubkey) = oracle_key(&env, 7);
        let signatures: Vec<(BytesN<32>, BytesN<64>)> = vec![
            &env,
            (pubkey.clone(), signed_reading(&env, &signing_key, &meter_id, 1, 1_704_067_200).1),
        ];

        for i in 1u64..=3u64 {
            let ts = 1_704_067_200u64 + i;
            let (_hash, sig, _) = signed_reading(&env, &signing_key, &meter_id, i, ts);
            let sigs = vec![&env, (pubkey.clone(), sig)];
            protocol.oracle_client.submit_reading(
                &meter_id,
                &i,
                &ts,
                &sigs,
                &0u32,
                &2026u32,
                &make_uri(&env),
            );
        }

        let retirer = protocol.generator.clone();
        let claim_ids = Vec::from_array(&env, [1u64, 2u64, 3u64]);
        protocol.retirement_client.set_retirement_fee(&2_000_000i128);
        let receipt_id = protocol.retirement_client.retire(&retirer, &claim_ids, &make_claim(&env));
        let receipt = protocol.retirement_client.get_retirement_receipt(&receipt_id);

        assert_eq!(receipt.total_mwh, 3);
        assert_eq!(receipt.retirer, retirer);

        let verified = protocol.retirement_client.verify_retirement(&1u64);
        assert_eq!(verified, Some(receipt_id.clone()));

        let not_retired = protocol.retirement_client.verify_retirement(&999u64);
        assert_eq!(not_retired, None);
    }

    #[test]
    fn test_auto_match_orders() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let buyer = Address::generate(&env);

        let meter_id = make_bytes_n(&env, 11);
        let (signing_key, pubkey) = oracle_key(&env, 7);
        let (_reading_hash, signature, _) =
            signed_reading(&env, &signing_key, &meter_id, 1, 1_704_067_200);
        let signatures: Vec<(BytesN<32>, BytesN<64>)> = vec![&env, (pubkey, signature)];

        let token_id = protocol.oracle_client.submit_reading(
            &meter_id,
            &1u64,
            &1_704_067_200u64,
            &signatures,
            &0u32,
            &2026u32,
            &make_uri(&env),
        );

        protocol.rec_client.transfer(&protocol.generator, &buyer, &token_id);

        protocol.market_client.place_order(
            &buyer,
            &OrderSide::Sell,
            &5_000_000i128,
            &1u64,
            &OrderRestriction::None,
            &Some(2026u32),
        );
        protocol.market_client.place_order(
            &protocol.buyer,
            &OrderSide::Buy,
            &5_000_000i128,
            &1u64,
            &OrderRestriction::None,
            &Some(2026u32),
        );

        let matched = protocol.market_client.auto_match();
        assert_eq!(matched, 1);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #14)")]
    fn test_marketplace_pause_blocks_cfd_liquidation() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let producer = Address::generate(&env);
        let offtaker = Address::generate(&env);
        let strike = 40_000_000i128;
        let qty = 5_000u64;
        let now = env.ledger().timestamp();
        let expiry = now + 1;
        let collateral = (qty as i128) * strike * 1_500 / 10_000;

        let pos_id =
            protocol
                .market_client
                .open_cfd(&producer, &strike, &qty, &expiry, &collateral);
        protocol
            .market_client
            .accept_cfd(&offtaker, &pos_id, &collateral);

        protocol.market_client.pause();
        protocol.market_client.liquidate(&protocol.bot, &pos_id);
    }

    #[test]
    fn test_set_verifier_and_verify_retirement() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let verifier = Address::generate(&env);

        assert!(!protocol.retirement_client.is_verifier(&verifier));
        protocol.retirement_client.set_verifier(&verifier, &true);
        assert!(protocol.retirement_client.is_verifier(&verifier));
        protocol.retirement_client.set_verifier(&verifier, &false);
        assert!(!protocol.retirement_client.is_verifier(&verifier));
    }

    #[test]
    fn test_full_cfd_settle_after_price_movement() {
        let env = Env::default();
        let protocol = setup_protocol(&env);
        let producer = Address::generate(&env);
        let offtaker = Address::generate(&env);
        let strike = 40_000_000i128;
        let qty = 5_000u64;
        let now = env.ledger().timestamp();
        let expiry = now + 1;
        let collateral = (qty as i128) * strike * 1_500 / 10_000;

        let pos_id =
            protocol
                .market_client
                .open_cfd(&producer, &strike, &qty, &expiry, &collateral);
        protocol
            .market_client
            .accept_cfd(&offtaker, &pos_id, &collateral);

        protocol.env.ledger().set_timestamp(now + 2);
        protocol.oracle_client.set_price(&55_000_000i128);
        protocol.market_client.settle_cfd(&offtaker, &pos_id);

        let position = protocol.market_client.get_cfd(&pos_id);
        assert_eq!(position.state, PositionState::Settled);
        assert_eq!(position.mtm_value, 75_000_000_000i128);
    }

    #[test]
    fn test_burn_unauthorized_caller_rejected() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let admin = Address::generate(&env);
        let minter = Address::generate(&env);
        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);

        let rec_id = env.register(RecTokenContract, (&admin,));
        let rec_client = RecTokenContractClient::new(&env, &rec_id);
        rec_client.authorize_minter(&minter);

        let token_id = rec_client.mint(
            &owner,
            &make_bytes_n(&env, 1),
            &1_704_067_200u64,
            &FuelType::Solar,
            &2026u32,
            &make_uri(&env),
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rec_client.burn(&attacker, &token_id);
        }));
        assert!(result.is_err());
    }
}
