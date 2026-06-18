use soroban_sdk::{Env, Vec};

use crate::storage::*;
use crate::types::*;

pub fn buy_orders(env: &Env) -> Vec<Order> {
    let counter = read_order_counter(env);
    let mut orders: Vec<Order> = Vec::new(env);

    for id in 1..=counter {
        if has_order(env, id) {
            let order = read_order(env, id);
            if order.side == OrderSide::Buy
                && order.status == OrderStatus::Open
                && order.remaining_qty > 0
            {
                orders.push_back(order);
            }
        }
    }

    let n = orders.len();
    if n <= 1 {
        return orders;
    }

    let mut sorted: Vec<Order> = Vec::new(env);
    while !orders.is_empty() {
        let mut best_idx = 0u32;
        let mut best_price = orders.get(0).unwrap().price;
        let mut best_ts = orders.get(0).unwrap().timestamp;
        for i in 1..orders.len() {
            let cur = orders.get(i).unwrap();
            if cur.price > best_price || (cur.price == best_price && cur.timestamp < best_ts) {
                best_price = cur.price;
                best_ts = cur.timestamp;
                best_idx = i;
            }
        }
        let best = orders.get(best_idx).unwrap();
        orders.remove(best_idx);
        sorted.push_back(best);
    }
    sorted
}

pub fn sell_orders(env: &Env) -> Vec<Order> {
    let counter = read_order_counter(env);
    let mut orders: Vec<Order> = Vec::new(env);

    for id in 1..=counter {
        if has_order(env, id) {
            let order = read_order(env, id);
            if order.side == OrderSide::Sell
                && order.status == OrderStatus::Open
                && order.remaining_qty > 0
            {
                orders.push_back(order);
            }
        }
    }

    let n = orders.len();
    if n <= 1 {
        return orders;
    }

    let mut sorted: Vec<Order> = Vec::new(env);
    while !orders.is_empty() {
        let mut best_idx = 0u32;
        let mut best_price = orders.get(0).unwrap().price;
        let mut best_ts = orders.get(0).unwrap().timestamp;
        for i in 1..orders.len() {
            let cur = orders.get(i).unwrap();
            if cur.price < best_price || (cur.price == best_price && cur.timestamp < best_ts) {
                best_price = cur.price;
                best_ts = cur.timestamp;
                best_idx = i;
            }
        }
        let best = orders.get(best_idx).unwrap();
        orders.remove(best_idx);
        sorted.push_back(best);
    }
    sorted
}

pub fn best_bid(env: &Env) -> Option<Order> {
    let buys = buy_orders(env);
    buys.first()
}

pub fn best_ask(env: &Env) -> Option<Order> {
    let sells = sell_orders(env);
    sells.first()
}
