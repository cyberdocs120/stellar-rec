use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{MarketplaceContract, MarketplaceContractClient, OrderSide, OrderStatus};

macro_rules! setup {
    ($env:ident, $client:ident, $admin:ident) => {
        let $env = Env::default();
        let $admin = Address::generate(&$env);
        $env.mock_all_auths();
        let contract_id = $env.register(MarketplaceContract, (&$admin,));
        let $client = MarketplaceContractClient::new(&$env, &contract_id);
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
