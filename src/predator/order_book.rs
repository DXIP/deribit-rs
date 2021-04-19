use crate::models::subscription::channels::{BookData, Delta, OrderBookDelta};

#[derive(Debug)]
pub struct OrderBookEntry {
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug)]
pub struct OrderBook {
    pub ask: f64, //min ask
    pub bid: f64, //max bid
    pub spread: f64,

    asks: Vec<OrderBookEntry>,
    bids: Vec<OrderBookEntry>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            ask: 999999999999f64,
            bid: 0f64,
            spread: 0f64,
            asks: vec![],
            bids: vec![],
        }
    }

    pub fn update(&mut self, book_data: &BookData) {
        for ask in book_data.asks.iter() {
            match ask.0 {
                Delta::New => {
                    self.asks.push(OrderBookEntry {
                        price: ask.1,
                        volume: ask.2,
                    });
                    if self.ask > ask.1 {
                        self.ask = ask.1;
                    }
                }
                Delta::Change => (),
                Delta::Delete => (),
            }
        }
        for bid in book_data.bids.iter() {
            match bid.0 {
                Delta::New => {
                    self.bids.push(OrderBookEntry {
                        price: bid.1,
                        volume: bid.2,
                    });
                    if self.bid < bid.1 {
                        self.bid = bid.1;
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
    assert_eq!(3.50f64, order_book.ask);
    assert_eq!(3.00f64, order_book.bid);
    assert_eq!(0.50f64, order_book.spread);
}
