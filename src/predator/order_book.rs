use crate::models::subscription::channels::{BookData, Delta, OrderBookDelta};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct OrderBook {
    pub ask: Decimal, //min ask
    pub bid: Decimal, //max bid
    pub spread: Decimal,

    pub asks: BTreeMap<Decimal /*price*/, f64 /*volume*/>,
    pub bids: BTreeMap<Decimal, f64>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            ask: Decimal::max_value(),
            bid: dec!(0.0),
            spread: dec!(0.0),
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
        }
    }

    pub fn update(&mut self, book_data: &BookData) {
        for ask in book_data.asks.iter() {
            let price = (ask.1 * 100.0).round() as i64;
            match ask.0 {
                Delta::New => {
                    let price_decimal = Decimal::new(price, 2);
                    self.asks.insert(price_decimal, ask.2);
                    if self.ask > price_decimal {
                        self.ask = price_decimal;
                    }
                }
                Delta::Change => {
                    let price_decimal = Decimal::new(price, 2);
                    self.asks.insert(price_decimal, ask.2);
                }
                Delta::Delete => {
                    let price_decimal = Decimal::new(price, 2);
                    self.asks.remove(&price_decimal);
                    if let Some(x) = self.asks.keys().min() {
                        self.ask = *x;
                    }
                }
            }
        }
        for bid in book_data.bids.iter() {
            let price = (bid.1 * 100.0).round() as i64;
            match bid.0 {
                Delta::New => {
                    let price_decimal = Decimal::new(price, 2);
                    self.bids.insert(price_decimal, bid.2);
                    if self.bid < price_decimal {
                        self.bid = price_decimal;
                    }
                }
                Delta::Change => {
                    let price_decimal = Decimal::new(price, 2);
                    self.bids.insert(price_decimal, bid.2);
                }
                Delta::Delete => {
                    let price_decimal = Decimal::new(price, 2);
                    self.bids.remove(&price_decimal);
                    if let Some(x) = self.bids.keys().max() {
                        self.bid = *x;
                    }
                }
            }
        }
        self.spread = self.ask - self.bid;
    }
}

#[test]
fn test_order_book_update() {
    let mut order_book = OrderBook::new();

    let book_data = BookData {
        asks: vec![OrderBookDelta(
            Delta::New,
            3.50f64, /*price*/
            3.50f64, /*amount*/
        )],
        bids: vec![OrderBookDelta(Delta::New, 3.00f64, 3.00f64)],
        change_id: 1231i64,
        instrument_name: String::from("BTC"),
        prev_change_id: None,
        timestamp: 23424u64,
    };
    order_book.update(&book_data);
    assert_eq!(dec!(3.50), order_book.ask);
    assert_eq!(dec!(3.00), order_book.bid);
    assert_eq!(dec!(0.50), order_book.spread);
}

#[test]
fn test_order_book_update_changes() {
    let mut order_book = OrderBook::new();

    let initial_book_data = BookData {
        asks: vec![OrderBookDelta(
            Delta::New,
            3.50f64, /*price*/
            3f64,    /*amount*/
        )],
        bids: vec![OrderBookDelta(Delta::New, 3.00f64, 3f64)],
        change_id: 1231i64,
        instrument_name: String::from("BTC"),
        prev_change_id: None,
        timestamp: 23424u64,
    };
    order_book.update(&initial_book_data);
    assert_eq!(dec!(3.50), order_book.ask);
    assert_eq!(dec!(3.00), order_book.bid);
    assert_eq!(dec!(0.50), order_book.spread);

    let update_book_data = BookData {
        asks: vec![OrderBookDelta(
            Delta::Change,
            3.50f64, /*price*/
            8f64,    /*amount*/
        )],
        bids: vec![OrderBookDelta(Delta::Change, 3.00f64, 5f64)],
        change_id: 1231i64,
        instrument_name: String::from("BTC"),
        prev_change_id: None,
        timestamp: 23424u64,
    };
    order_book.update(&update_book_data);
    assert_eq!(8f64, order_book.asks[&dec!(3.50)]);
    assert_eq!(5f64, order_book.bids[&dec!(3.00)]);
    assert_eq!(dec!(0.50), order_book.spread);
}

#[test]
fn test_order_book_update_changes_deletes() {
    let mut order_book = OrderBook::new();

    let initial_book_data = BookData {
        asks: vec![
            OrderBookDelta(Delta::New, 3.50f64 /*price*/, 3f64 /*amount*/),
            OrderBookDelta(Delta::New, 4.50f64 /*price*/, 6f64 /*amount*/),
        ],
        bids: vec![
            OrderBookDelta(Delta::New, 3.00f64, 3f64),
            OrderBookDelta(Delta::New, 2.30f64, 4f64),
        ],
        change_id: 1231i64,
        instrument_name: String::from("BTC"),
        prev_change_id: None,
        timestamp: 23424u64,
    };
    order_book.update(&initial_book_data);
    println!("first={:?}", order_book);
    assert_eq!(dec!(3.50), order_book.ask);
    assert_eq!(dec!(3.00), order_book.bid);
    assert_eq!(dec!(0.50), order_book.spread);

    let update_book_data = BookData {
        asks: vec![
            OrderBookDelta(Delta::Change, 4.50f64 /*price*/, 8f64 /*amount*/),
            OrderBookDelta(Delta::Delete, 3.50f64 /*price*/, 0f64 /*amount*/),
        ],
        bids: vec![
            OrderBookDelta(Delta::Change, 2.30f64, 5f64),
            OrderBookDelta(Delta::Delete, 3.00f64, 5f64),
        ],
        change_id: 1231i64,
        instrument_name: String::from("BTC"),
        prev_change_id: None,
        timestamp: 23424u64,
    };
    order_book.update(&update_book_data);
    println!("second={:?}", order_book);
    assert_eq!(8f64, order_book.asks[&dec!(4.50)]);
    assert_eq!(5f64, order_book.bids[&dec!(2.30)]);
    assert_eq!(dec!(4.50), order_book.ask);
    assert_eq!(dec!(2.30), order_book.bid);
    assert_eq!(dec!(2.20), order_book.spread);
}
