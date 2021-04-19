use crate::models::subscription::channels::{BookData, Delta, OrderBookDelta};
use rust_decimal_macros::dec;
use rust_decimal::prelude::*;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct OrderBook {
    pub ask: Decimal, //min ask
    pub bid: Decimal, //max bid
    pub spread: Decimal,

    asks: BTreeMap<Decimal /*price*/, f64 /*volume*/>,
    bids: BTreeMap<Decimal, f64>,
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
            let price = (ask.1*100f64).trunc() as i64;
            match ask.0 {
                Delta::New => {
                    let price_decimal = Decimal::new(price, 2);
                    self.asks.insert(price_decimal, ask.2);
                    if self.ask > price_decimal {
                        self.ask = price_decimal;
                    }
                }
                Delta::Change => (),
                Delta::Delete => (),
            }
        }
        for bid in book_data.bids.iter() {
            let price = (bid.1*100f64).trunc() as i64;
            match bid.0 {
                Delta::New => {
                    let price_decimal = Decimal::new(price, 2);
                    self.bids.insert(price_decimal, bid.2);
                    if self.bid < price_decimal {
                        self.bid = price_decimal;
                    }
                }
                Delta::Change => (),
                Delta::Delete => (),
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
