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

const INITIAL_MARGIN_BPS: u32 = 1500; // 15%
const MAINTENANCE_MARGIN_BPS: u32 = 1000; // 10%

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

    pub fn set_oracle_handler(env: Env, addr: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        write_oracle_handler(&env, &addr);
    }

    pub fn oracle_handler(env: Env) -> Address {
        read_oracle_handler(&env)
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

    // ---------- CfD Functions ----------

    pub fn open_cfd(
        env: Env,
        caller: Address,
        strike: i128,
        qty: u64,
        expiry: u64,
        collateral: i128,
    ) -> u64 {
        caller.require_auth();

        if strike <= 0 || qty == 0 || expiry <= env.ledger().timestamp() || collateral <= 0 {
            panic_with_error!(&env, MarketError::InvalidQuantity);
        }

        let notional = (qty as i128) * strike;
        let min_collateral = notional * (INITIAL_MARGIN_BPS as i128) / 10000;
        if collateral < min_collateral {
            panic_with_error!(&env, MarketError::InsufficientCollateral);
        }

        let usdc_id = read_usdc_token(&env);
        let _: () = env.invoke_contract(
            &usdc_id,
            &symbol_short!("xfer"),
            (caller.clone(), env.current_contract_address(), collateral).into_val(&env),
        );

        let counter = read_position_counter(&env) + 1;
        write_position_counter(&env, counter);

        let position = CfDPosition {
            position_id: counter,
            counterparty_a: caller,
            counterparty_b: None,
            strike_price: strike,
            quantity: qty,
            settlement_date: expiry,
            collateral_a: collateral,
            collateral_b: 0,
            maintenance_margin_bps: MAINTENANCE_MARGIN_BPS,
            state: PositionState::Pending,
            last_mtm_timestamp: 0,
            mtm_value: 0,
        };
        write_position(&env, counter, &position);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("cfdp")),
            (counter, position.counterparty_a, strike, qty, expiry),
        );

        counter
    }

    pub fn accept_cfd(env: Env, caller: Address, position_id: u64, collateral: i128) {
        caller.require_auth();

        if !has_position(&env, position_id) {
            panic_with_error!(&env, MarketError::PositionNotFound);
        }

        let mut position = read_position(&env, position_id);
        if position.state != PositionState::Pending {
            panic_with_error!(&env, MarketError::PositionNotActive);
        }
        if position.counterparty_a == caller {
            panic_with_error!(&env, MarketError::Unauthorized);
        }

        if collateral <= 0 {
            panic_with_error!(&env, MarketError::InsufficientCollateral);
        }

        let notional = (position.quantity as i128) * position.strike_price;
        let min_collateral = notional * (INITIAL_MARGIN_BPS as i128) / 10000;
        if collateral < min_collateral {
            panic_with_error!(&env, MarketError::InsufficientCollateral);
        }

        let usdc_id = read_usdc_token(&env);
        let _: () = env.invoke_contract(
            &usdc_id,
            &symbol_short!("xfer"),
            (caller.clone(), env.current_contract_address(), collateral).into_val(&env),
        );

        let counterparty_b = caller.clone();
        position.counterparty_b = Some(caller);
        position.collateral_b = collateral;
        position.state = PositionState::Active;
        write_position(&env, position_id, &position);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("cfdo")),
            (
                position_id,
                position.counterparty_a,
                counterparty_b,
                position.strike_price,
                position.quantity,
                position.settlement_date,
            ),
        );
    }

    pub fn add_collateral(env: Env, caller: Address, position_id: u64, amount: i128) {
        caller.require_auth();

        if !has_position(&env, position_id) {
            panic_with_error!(&env, MarketError::PositionNotFound);
        }

        let mut position = read_position(&env, position_id);
        if position.state != PositionState::Active {
            panic_with_error!(&env, MarketError::PositionNotActive);
        }

        if amount <= 0 {
            panic_with_error!(&env, MarketError::InsufficientCollateral);
        }

        if caller == position.counterparty_a {
            position.collateral_a += amount;
        } else if Some(caller.clone()) == position.counterparty_b {
            position.collateral_b += amount;
        } else {
            panic_with_error!(&env, MarketError::Unauthorized);
        }

        let usdc_id = read_usdc_token(&env);
        let _: () = env.invoke_contract(
            &usdc_id,
            &symbol_short!("xfer"),
            (caller.clone(), env.current_contract_address(), amount).into_val(&env),
        );

        write_position(&env, position_id, &position);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("cfda")),
            (position_id, caller, amount),
        );
    }

    pub fn remove_collateral(env: Env, caller: Address, position_id: u64, amount: i128) {
        caller.require_auth();

        if !has_position(&env, position_id) {
            panic_with_error!(&env, MarketError::PositionNotFound);
        }

        let mut position = read_position(&env, position_id);
        if position.state != PositionState::Active {
            panic_with_error!(&env, MarketError::PositionNotActive);
        }

        if amount <= 0 {
            panic_with_error!(&env, MarketError::InsufficientCollateral);
        }

        if caller == position.counterparty_a {
            if amount > position.collateral_a {
                panic_with_error!(&env, MarketError::InsufficientCollateral);
            }
            let remaining = position.collateral_a - amount;
            let notional = (position.quantity as i128) * position.strike_price;
            let min_collateral = notional * (INITIAL_MARGIN_BPS as i128) / 10000;
            if remaining < min_collateral {
                panic_with_error!(&env, MarketError::CollateralBelowMaintenance);
            }
            position.collateral_a = remaining;
        } else if Some(caller.clone()) == position.counterparty_b {
            if amount > position.collateral_b {
                panic_with_error!(&env, MarketError::InsufficientCollateral);
            }
            let remaining = position.collateral_b - amount;
            let notional = (position.quantity as i128) * position.strike_price;
            let min_collateral = notional * (INITIAL_MARGIN_BPS as i128) / 10000;
            if remaining < min_collateral {
                panic_with_error!(&env, MarketError::CollateralBelowMaintenance);
            }
            position.collateral_b = remaining;
        } else {
            panic_with_error!(&env, MarketError::Unauthorized);
        }

        let usdc_id = read_usdc_token(&env);
        let _: () = env.invoke_contract(
            &usdc_id,
            &symbol_short!("xfer"),
            (env.current_contract_address(), caller.clone(), amount).into_val(&env),
        );

        write_position(&env, position_id, &position);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("cfdr")),
            (position_id, caller, amount),
        );
    }

    pub fn get_cfd(env: Env, position_id: u64) -> CfDPosition {
        if !has_position(&env, position_id) {
            panic_with_error!(&env, MarketError::PositionNotFound);
        }
        read_position(&env, position_id)
    }

    pub fn settle_cfd(env: Env, caller: Address, position_id: u64) {
        caller.require_auth();

        if !has_position(&env, position_id) {
            panic_with_error!(&env, MarketError::PositionNotFound);
        }

        let mut position = read_position(&env, position_id);
        if position.state != PositionState::Active {
            panic_with_error!(&env, MarketError::PositionNotActive);
        }

        if env.ledger().timestamp() < position.settlement_date {
            panic_with_error!(&env, MarketError::PositionNotActive); // Using NotActive as generic "Not Ready"
        }

        // Fetch spot price from Oracle Handler
        let oracle_handler = read_oracle_handler(&env);
        let spot_price: i128 = env.invoke_contract(
            &oracle_handler,
            &symbol_short!("get_price"),
            ().into_val(&env),
        );

        if spot_price <= 0 {
            panic_with_error!(&env, MarketError::PriceMismatch);
        }

        let strike = position.strike_price;
        let qty = position.quantity as i128;
        let diff = spot_price - strike;
        let payoff = diff * qty;

        let usdc_id = read_usdc_token(&env);
        let mut final_a = position.collateral_a;
        let mut final_b = position.collateral_b;

        if payoff > 0 {
            // A pays B
            if payoff > final_a {
                // A is bankrupt
                final_b += final_a;
                final_a = 0;
            } else {
                final_a -= payoff;
                final_b += payoff;
            }
        } else if payoff < 0 {
            // B pays A
            let abs_payoff = -payoff;
            if abs_payoff > final_b {
                // B is bankrupt
                final_a += final_b;
                final_b = 0;
            } else {
                final_b -= abs_payoff;
                final_a += abs_payoff;
            }
        }

        // Return collateral to parties
        if final_a > 0 {
            transfer_usdc(&env, &usdc_id, &env.current_contract_address(), &position.counterparty_a, final_a);
        }
        if final_b > 0 {
            let b_addr = position.counterparty_b.clone().unwrap();
            transfer_usdc(&env, &usdc_id, &env.current_contract_address(), &b_addr, final_b);
        }

        position.state = PositionState::Settled;
        position.last_mtm_timestamp = env.ledger().timestamp();
        position.mtm_value = payoff;
        write_position(&env, position_id, &position);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("cfds")),
            (position_id, spot_price, payoff),
        );
    }

    pub fn liquidate(env: Env, caller: Address, position_id: u64) {
        caller.require_auth();

        if !has_position(&env, position_id) {
            panic_with_error!(&env, MarketError::PositionNotFound);
        }

        let mut position = read_position(&env, position_id);
        if position.state != PositionState::Active {
            panic_with_error!(&env, MarketError::PositionNotActive);
        }

        // Fetch spot price from Oracle Handler
        let oracle_handler = read_oracle_handler(&env);
        let spot_price: i128 = env.invoke_contract(
            &oracle_handler,
            &symbol_short!("get_price"),
            ().into_val(&env),
        );

        let strike = position.strike_price;
        let qty = position.quantity as i128;
        let diff = spot_price - strike;
        let payoff = diff * qty;

        let notional = (position.quantity as i128) * position.strike_price;
        let mm_bps = position.maintenance_margin_bps as i128;
        let mm_amount = notional * mm_bps / 10000;

        // Current value of A's collateral after unrealized P&L
        let current_val_a = position.collateral_a - payoff;
        let current_val_b = position.collateral_b + payoff;

        let mut liquidatable = false;
        if current_val_a < mm_amount || current_val_b < mm_amount {
            liquidatable = true;
        }

        if !liquidatable {
            panic_with_error!(&env, MarketError::InsufficientCollateral); // Not liquidatable yet
        }

        // Settlement logic (same as settle_cfd but marks as liquidated)
        let usdc_id = read_usdc_token(&env);
        let mut final_a = position.collateral_a;
        let mut final_b = position.collateral_b;

        if payoff > 0 {
            if payoff > final_a {
                final_b += final_a;
                final_a = 0;
            } else {
                final_a -= payoff;
                final_b += payoff;
            }
        } else if payoff < 0 {
            let abs_payoff = -payoff;
            if abs_payoff > final_b {
                final_a += final_b;
                final_b = 0;
            } else {
                final_b -= abs_payoff;
                final_a += abs_payoff;
            }
        }

        // In liquidation, the liquidator might get a small reward?
        // For now, just settle it.
        if final_a > 0 {
            transfer_usdc(&env, &usdc_id, &env.current_contract_address(), &position.counterparty_a, final_a);
        }
        if final_b > 0 {
            let b_addr = position.counterparty_b.clone().unwrap();
            transfer_usdc(&env, &usdc_id, &env.current_contract_address(), &b_addr, final_b);
        }

        position.state = PositionState::Liquidated;
        position.last_mtm_timestamp = env.ledger().timestamp();
        position.mtm_value = payoff;
        write_position(&env, position_id, &position);

        env.events().publish(
            (symbol_short!("mkt"), symbol_short!("cfdl")),
            (position_id, spot_price, payoff),
        );
    }
}
