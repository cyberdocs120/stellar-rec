#![no_std]

mod storage;
mod types;
mod errors;
mod order_book;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, Address, Env, IntoVal, Symbol, Vec};

use errors::MarketError;
use storage::*;
use types::*;

fn transfer_rec(env: &Env, rec_id: &Address, from: &Address, to: &Address, token_id: u64) {
    let _: () = env.invoke_contract(
        rec_id,
        &Symbol::new(env, "transfer"),
        (from.clone(), to.clone(), token_id).into_val(env),
    );
}

fn transfer_usdc(env: &Env, usdc_id: &Address, from: &Address, to: &Address, amount: i128) {
    let _: () = env.invoke_contract(
        usdc_id,
        &symbol_short!("xfer"),
        (from.clone(), to.clone(), amount).into_val(env),
    );
}

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

    pub fn match_orders(env: Env, buy_id: u64, sell_id: u64) -> (u64, i128, i128) {
        if !has_order(&env, buy_id) || !has_order(&env, sell_id) {
            panic_with_error!(&env, MarketError::OrderNotFound);
        }

        let mut buy = read_order(&env, buy_id);
        let mut sell = read_order(&env, sell_id);

        if buy.status != OrderStatus::Open || sell.status != OrderStatus::Open {
            panic_with_error!(&env, MarketError::OrderFilled);
        }
        if buy.remaining_qty == 0 || sell.remaining_qty == 0 {
            panic_with_error!(&env, MarketError::InvalidQuantity);
        }
        if buy.side != OrderSide::Buy || sell.side != OrderSide::Sell {
            panic_with_error!(&env, MarketError::PriceMismatch);
        }
        if buy.price < sell.price {
            panic_with_error!(&env, MarketError::PriceMismatch);
        }

        if let (Some(bv), Some(sv)) = (buy.vintage_filter, sell.vintage_filter) {
            if bv != sv {
                panic_with_error!(&env, MarketError::VintageMismatch);
            }
        }

        let fill_qty = if buy.remaining_qty < sell.remaining_qty {
            buy.remaining_qty
        } else {
            sell.remaining_qty
        };
        let fill_price = sell.price;
        let fill_value = fill_price * (fill_qty as i128);
        let fee_rate = read_fee_rate(&env);
        let fee = fill_value * (fee_rate as i128) / 10000;

        let rec_id = read_rec_token(&env);
        let usdc_id = read_usdc_token(&env);
        let vault = read_fee_vault(&env);

        transfer_rec(&env, &rec_id, &sell.trader, &buy.trader, fill_qty);
        transfer_usdc(&env, &usdc_id, &buy.trader, &sell.trader, fill_value - fee);
        transfer_usdc(&env, &usdc_id, &buy.trader, &vault, fee);

        buy.remaining_qty -= fill_qty;
        sell.remaining_qty -= fill_qty;

        if buy.remaining_qty == 0 {
            buy.status = OrderStatus::Filled;
        }
        if sell.remaining_qty == 0 {
            sell.status = OrderStatus::Filled;
        }

        write_order(&env, buy_id, &buy);
        write_order(&env, sell_id, &sell);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("matc")),
            (buy_id, sell_id, fill_qty, fill_price, fee),
        );

        (fill_qty, fill_price, fee)
    }

    pub fn auto_match(env: Env) -> u32 {
        let mut count = 0u32;
        loop {
            let bid = order_book::best_bid(&env);
            let ask = order_book::best_ask(&env);
            let should_match = match (&bid, &ask) {
                (Some(b), Some(a)) => {
                    b.price >= a.price && b.remaining_qty > 0 && a.remaining_qty > 0
                }
                _ => false,
            };
            if should_match {
                let b_id = bid.unwrap().order_id;
                let a_id = ask.unwrap().order_id;
                Self::match_orders(env.clone(), b_id, a_id);
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}
