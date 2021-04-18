use crate::models::subscription::channels::{BookData, Delta, OrderBookDelta};

#[derive(Debug)]
pub struct OrderBookEntry {
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug)]
pub struct OrderBook {
    pub ask: f64, //max ask
    pub bid: f64, //min bid
    pub spread: f64,

    asks: Vec<OrderBookEntry>,
    bids: Vec<OrderBookEntry>,
}

impl OrderBook {

    pub fn new() -> OrderBook {
        OrderBook {
            ask: 0f64,
            bid: 999999999999.99f64,
            spread: 0f64,
            asks: vec![],
            bids: vec![],
        }
    }

    pub fn load(&mut self, book_data: &BookData) {
        for ask in book_data.asks.clone() {
            self.asks.push(OrderBookEntry {
                price: ask.1,
                volume: ask.2,
            });
            if self.ask < ask.1 {
                self.ask = ask.1;
            }
        }
        for bid in book_data.bids.clone() {
            self.bids.push(OrderBookEntry {
                price: bid.1,
                volume: bid.2,
            });
            if self.bid > bid.1 {
                self.bid = bid.1;
            }
        }
        self.spread = self.bid - self.ask;
    }

    pub fn update(&mut self, book_data: &BookData) {
        for ask in book_data.asks.clone() {}
    }
}

#[test]
fn test_order_book_update() {
    let mut order_book = OrderBook::new();

    let book_data = BookData {
        asks: vec![OrderBookDelta(
            Delta::New,
            2.50f64, /*price*/
            3.50f64, /*amount*/
        )],
        bids: vec![OrderBookDelta(Delta::New, 3.00f64, 3.00f64)],
        change_id: 1231i64,
        instrument_name: String::from("BTC"),
        prev_change_id: None,
        timestamp: 23424u64,
    };
    order_book.load(&book_data);
    assert_eq!(2.50f64, order_book.ask);
    assert_eq!(3.00f64, order_book.bid);
    assert_eq!(0.50f64, order_book.spread);
}
