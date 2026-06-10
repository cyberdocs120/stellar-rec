use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{MarketplaceContract, MarketplaceContractClient, OrderSide, OrderStatus};

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

mod mock_rec {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    #[contract]
    pub struct MockRec;

    #[contractimpl]
    impl MockRec {
        pub fn __constructor(_env: Env, _admin: Address) {}

        pub fn transfer(_env: Env, _from: Address, _to: Address, _token_id: u64) {}
    }
}

macro_rules! setup {
    ($env:ident, $client:ident, $admin:ident) => {
        let $env = Env::default();
        let $admin = Address::generate(&$env);
        $env.mock_all_auths();
        let contract_id = $env.register(MarketplaceContract, (&$admin,));
        let $client = MarketplaceContractClient::new(&$env, &contract_id);
    };
}

macro_rules! setup_with_tokens {
    ($env:ident, $client:ident, $admin:ident, $usdc_id:ident, $rec_id:ident, $vault:ident) => {
        let $env = Env::default();
        let $admin = Address::generate(&$env);
        let $vault = Address::generate(&$env);
        $env.mock_all_auths();

        let usdc_contract_id = $env.register(mock_usdc::MockUsdc, (&$admin,));
        let $usdc_id = usdc_contract_id.clone();
        let $usdc_id: Address = $usdc_id;

        let rec_contract_id = $env.register(mock_rec::MockRec, (&$admin,));
        let $rec_id = rec_contract_id.clone();
        let $rec_id: Address = $rec_id;

        let contract_id = $env.register(MarketplaceContract, (&$admin,));
        let $client = MarketplaceContractClient::new(&$env, &contract_id);

        $client.set_rec_token(&$rec_id);
        $client.set_usdc_token(&$usdc_id);
        $client.set_fee_vault(&$vault);
    };
}

#[test]
fn test_place_buy_order() {
    setup!(env, client, _admin);

    env.mock_all_auths();
    let trader = Address::generate(&env);
    let order_id = client.place_order(
        &trader,
        &OrderSide::Buy,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );

    assert_eq!(order_id, 1);
    let order = client.get_order(&order_id);
    assert_eq!(order.order_id, 1);
    assert_eq!(order.side, OrderSide::Buy);
    assert_eq!(order.price, 100_000_000);
    assert_eq!(order.initial_qty, 10);
    assert_eq!(order.remaining_qty, 10);
    assert_eq!(order.status, OrderStatus::Open);
}

#[test]
fn test_place_sell_order() {
    setup!(env, client, _admin);

    env.mock_all_auths();
    let trader = Address::generate(&env);
    let order_id = client.place_order(
        &trader,
        &OrderSide::Sell,
        &90_000_000i128,
        &5u64,
        &crate::OrderRestriction::None,
        &None,
    );

    assert_eq!(order_id, 1);
    let order = client.get_order(&order_id);
    assert_eq!(order.side, OrderSide::Sell);
}

#[test]
fn test_cancel_order() {
    setup!(env, client, _admin);

    env.mock_all_auths();
    let trader = Address::generate(&env);
    let order_id = client.place_order(
        &trader,
        &OrderSide::Buy,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );

    client.cancel_order(&trader, &order_id);
    let order = client.get_order(&order_id);
    assert_eq!(order.status, OrderStatus::Cancelled);
}

#[test]
fn test_best_bid_ask() {
    setup!(env, client, _admin);

    env.mock_all_auths();
    let trader = Address::generate(&env);
    client.place_order(
        &trader,
        &OrderSide::Buy,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &trader,
        &OrderSide::Sell,
        &90_000_000i128,
        &5u64,
        &crate::OrderRestriction::None,
        &None,
    );

    let best_bid = client.get_best_bid();
    assert!(best_bid.is_some());
    assert_eq!(best_bid.unwrap().price, 100_000_000);

    let best_ask = client.get_best_ask();
    assert!(best_ask.is_some());
    assert_eq!(best_ask.unwrap().price, 90_000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_cancel_nonexistent() {
    setup!(env, client, _admin);

    env.mock_all_auths();
    let caller = Address::generate(&env);
    client.cancel_order(&caller, &999u64);
}

#[test]
fn test_price_time_priority() {
    setup!(env, client, _admin);

    env.mock_all_auths();
    let trader = Address::generate(&env);

    client.place_order(
        &trader,
        &OrderSide::Buy,
        &95_000_000i128,
        &5u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &trader,
        &OrderSide::Buy,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &trader,
        &OrderSide::Buy,
        &98_000_000i128,
        &7u64,
        &crate::OrderRestriction::None,
        &None,
    );

    let buys = client.get_buy_orders();
    assert_eq!(buys.len(), 3);
    assert_eq!(buys.get(0).unwrap().price, 100_000_000);
    assert_eq!(buys.get(1).unwrap().price, 98_000_000);
    assert_eq!(buys.get(2).unwrap().price, 95_000_000);

    client.place_order(
        &trader,
        &OrderSide::Sell,
        &90_000_000i128,
        &5u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &trader,
        &OrderSide::Sell,
        &85_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &trader,
        &OrderSide::Sell,
        &88_000_000i128,
        &7u64,
        &crate::OrderRestriction::None,
        &None,
    );

    let sells = client.get_sell_orders();
    assert_eq!(sells.len(), 3);
    assert_eq!(sells.get(0).unwrap().price, 85_000_000);
    assert_eq!(sells.get(1).unwrap().price, 88_000_000);
    assert_eq!(sells.get(2).unwrap().price, 90_000_000);
}

// ---------- Day 8: Matching Engine Tests ----------

#[test]
fn test_match_orders_basic() {
    setup_with_tokens!(env, client, admin, usdc_id, rec_id, vault);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    client.place_order(
        &buyer,
        &OrderSide::Buy,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &seller,
        &OrderSide::Sell,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );

    let (fill_qty, fill_price, fee) = client.match_orders(&1u64, &2u64);
    // fill_value = 100_000_000 * 10 = 1_000_000_000
    // fee = 1_000_000_000 * 10 / 10000 = 1_000_000
    assert_eq!(fill_qty, 10);
    assert_eq!(fill_price, 100_000_000);
    assert_eq!(fee, 1_000_000);

    let buy = client.get_order(&1u64);
    let sell = client.get_order(&2u64);
    assert_eq!(buy.status, OrderStatus::Filled);
    assert_eq!(sell.status, OrderStatus::Filled);
    assert_eq!(buy.remaining_qty, 0);
    assert_eq!(sell.remaining_qty, 0);
}

#[test]
fn test_match_partial_fill() {
    setup_with_tokens!(env, client, admin, usdc_id, rec_id, vault);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    client.place_order(
        &buyer,
        &OrderSide::Buy,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &seller,
        &OrderSide::Sell,
        &100_000_000i128,
        &5u64,
        &crate::OrderRestriction::None,
        &None,
    );

    let (fill_qty, fill_price, _fee) = client.match_orders(&1u64, &2u64);
    assert_eq!(fill_qty, 5);
    assert_eq!(fill_price, 100_000_000);

    let buy = client.get_order(&1u64);
    let sell = client.get_order(&2u64);
    assert_eq!(buy.status, OrderStatus::Open);
    assert_eq!(buy.remaining_qty, 5);
    assert_eq!(sell.status, OrderStatus::Filled);
    assert_eq!(sell.remaining_qty, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_price_mismatch_rejected() {
    setup_with_tokens!(env, client, admin, usdc_id, rec_id, vault);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    client.place_order(
        &buyer,
        &OrderSide::Buy,
        &90_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &seller,
        &OrderSide::Sell,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );

    client.match_orders(&1u64, &2u64);
}

#[test]
fn test_auto_match_multiple() {
    setup_with_tokens!(env, client, admin, usdc_id, rec_id, vault);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    // Buy orders: $100 for 10, $99 for 5
    // Sell orders: $98 for 8, $100 for 7
    client.place_order(
        &buyer,
        &OrderSide::Buy,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &buyer,
        &OrderSide::Buy,
        &99_000_000i128,
        &5u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &seller,
        &OrderSide::Sell,
        &98_000_000i128,
        &8u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &seller,
        &OrderSide::Sell,
        &100_000_000i128,
        &7u64,
        &crate::OrderRestriction::None,
        &None,
    );

    let count = client.auto_match();
    assert!(count > 0);
}

#[test]
fn test_fee_deduction() {
    setup_with_tokens!(env, client, admin, usdc_id, rec_id, vault);

    // fee_rate defaults to 10 bps (0.1%)
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    client.place_order(
        &buyer,
        &OrderSide::Buy,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );
    client.place_order(
        &seller,
        &OrderSide::Sell,
        &100_000_000i128,
        &10u64,
        &crate::OrderRestriction::None,
        &None,
    );

    let (fill_qty, fill_price, fee) = client.match_orders(&1u64, &2u64);
    // fill_value = 100_000_000 * 10 = 1_000_000_000
    // fee = 1_000_000_000 * 10 / 10000 = 1_000_000
    assert_eq!(fee, 1_000_000);
    assert_eq!(fill_qty, 10);
    assert_eq!(fill_price, 100_000_000);
}
