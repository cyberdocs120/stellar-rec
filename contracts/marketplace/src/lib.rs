#![no_std]

mod storage;
mod types;
mod errors;
mod order_book;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, Address, Env, Vec};

use errors::MarketError;
use storage::*;
use types::*;

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        write_admin(&env, &admin);
        write_fee_rate(&env, 10);
        write_order_counter(&env, 0);
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

    pub fn set_fee_rate(env: Env, rate: u32) {
        let admin = read_admin(&env);
        admin.require_auth();
        if rate > 50 {
            panic_with_error!(&env, MarketError::FeeCapExceeded);
        }
        write_fee_rate(&env, rate);
    }

    pub fn fee_rate(env: Env) -> u32 {
        read_fee_rate(&env)
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

    pub fn place_order(
        env: Env,
        trader: Address,
        side: OrderSide,
        price: i128,
        qty: u64,
        restrictions: OrderRestriction,
        vintage_filter: Option<u32>,
    ) -> u64 {
        trader.require_auth();

        let counter = read_order_counter(&env) + 1;
        write_order_counter(&env, counter);

        let order = Order {
            order_id: counter,
            trader: trader.clone(),
            side,
            price,
            initial_qty: qty,
            remaining_qty: qty,
            timestamp: env.ledger().timestamp(),
            restrictions,
            vintage_filter,
            status: OrderStatus::Open,
        };
        write_order(&env, counter, &order);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("plac")),
            (counter, trader, side),
        );

        counter
    }

    pub fn cancel_order(env: Env, caller: Address, order_id: u64) {
        caller.require_auth();

        if !has_order(&env, order_id) {
            panic_with_error!(&env, MarketError::OrderNotFound);
        }

        let mut order = read_order(&env, order_id);
        if order.trader != caller {
            panic_with_error!(&env, MarketError::Unauthorized);
        }
        if order.status != OrderStatus::Open {
            panic_with_error!(&env, MarketError::OrderFilled);
        }

        order.status = OrderStatus::Cancelled;
        write_order(&env, order_id, &order);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("canc")),
            (order_id,),
        );
    }

    pub fn get_order(env: Env, order_id: u64) -> Order {
        if !has_order(&env, order_id) {
            panic_with_error!(&env, MarketError::OrderNotFound);
        }
        read_order(&env, order_id)
    }

    pub fn get_best_bid(env: Env) -> Option<Order> {
        order_book::best_bid(&env)
    }

    pub fn get_best_ask(env: Env) -> Option<Order> {
        order_book::best_ask(&env)
    }

    pub fn get_buy_orders(env: Env) -> Vec<Order> {
        order_book::buy_orders(&env)
    }

    pub fn get_sell_orders(env: Env) -> Vec<Order> {
        order_book::sell_orders(&env)
    }
}
