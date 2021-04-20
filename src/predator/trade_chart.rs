use rust_decimal::prelude::Decimal;
use std::collections::BTreeMap;

enum TickDirection {
    PlusTick = 0,
    ZeroPlusTick = 1,
    MinusTick = 2,
    ZeroMinusTick = 3,
}

enum Direction {
    Buy,
    Sell,
}
/*
trait Instrument {
    type Name;

    fn name_from_str(s: &str) -> Self::Name;
    fn name(&self) -> Self::Name;
}

struct DeribitFuture {}
impl Instrument for DeribitFuture {
    type Name = String;

    fn name_from_str(s: &str) -> Self::Name {
        String::from(s)
    }
    //_name: Self::Name;
    fn name(&self) -> Self::Name {
        "".to_string()
    }
}
struct DeribitOption {}
impl Instrument for DeribitOption {
    type Name = String;

    fn name_from_str(s: &str) -> Self::Name {
        String::from(s)
    }
    fn name(&self) -> Self::Name {
        "".to_string()
    }
}
*/
struct Trade {
    trade_seq: u64,
    trade_id: String,
    timestamp: u64,
    tick_direction: TickDirection,
    price: Decimal,
    mark_price: Decimal,
    instrument_name: String,
    index_price: Decimal,
    direction: Direction,
    amount: u64,
}

impl Trade {
    fn new() -> Trade {
        Trade {
            trade_seq: 0,
            trade_id: "0".to_string(),
            timestamp: 0,
            tick_direction: TickDirection::PlusTick,
            price: Decimal::new(0, 0),
            mark_price: Decimal::new(0, 0),
            instrument_name: "".to_string(),
            index_price: Decimal::new(0, 0),
            direction: Direction::Sell,
            amount: 0,
        }
    }
}

#[derive(Debug)]
struct Candle {
    pub duration: u64,
    pub start_timestamp: u64,

    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub cost: u64,
    pub volume: f64,
}

impl Candle {
    pub fn new() -> Candle {
        Candle {
            duration: 60,
            start_timestamp: 0,
            open: Decimal::new(0, 2),
            high: Decimal::new(0, 2),
            low: Decimal::new(0, 2),
            close: Decimal::new(0, 2),
            cost: 0,
            volume: 0.0,
        }
    }

    fn _align_timestamp(r: Resolution, timestamp: u64) -> u64 {
        (timestamp / 1000) / (r * 60) * (r * 60) * 1000
    }

    fn create(t: &Trade) -> Candle {
        Candle {
            duration: 60,
            start_timestamp: Self::_align_timestamp(60, t.timestamp),
            open: t.price,
            high: t.price,
            low: t.price,
            close: t.price,
            cost: t.amount,
            volume: 0.0,
        }
    }

    fn update(&mut self, t: &Trade) -> Result<(), Candle> {
        if self.start_timestamp + self.duration < t.timestamp {
            Err(Candle::create(&t))
        } else {
            if self.open == Decimal::new(0, 0) {
                self.open = t.price;
            }

            if self.low == Decimal::new(0, 0) || self.low > t.price {
                self.low = t.price;
            }

            if self.high < t.price {
                self.high = t.price;
            }
            self.close = t.price;

            self.cost = self.cost + t.amount;
            Ok(())
        }
    }
}

type TradeCandleChart = BTreeMap<u64, Candle>;
type Resolution = u64;

struct TradeLog {
    pub trade_log: Vec<Trade>,

    observers: BTreeMap<Resolution, TradeCandleChart>,
}

impl TradeLog {
    pub fn trade_chart(&mut self, r: &Resolution) -> &TradeCandleChart {
        self.observers.entry(*r).or_insert(BTreeMap::new())
    }

    pub fn new_trade(&mut self, t: Trade) -> () {
        //self.trade_log
        for (_r, chart) in self.observers.iter_mut() {
            if let Some(candle) = chart.values_mut().last() {
                candle.update(&t).expect("new candle case not handled");
                //TODO
            } else {
                println!("ERROR");
            }
        }
    }
}

#[test]
fn test_candle_create() {
    let mut trade1 = Trade::new();
    trade1.timestamp = 1;
    trade1.price = Decimal::new(10, 0);
    trade1.amount = 10;
    let mut c = Candle::create(&trade1);

    assert_eq!(Decimal::new(10, 0), c.open);
    assert_eq!(Decimal::new(10, 0), c.high);
    assert_eq!(Decimal::new(10, 0), c.low);
    assert_eq!(Decimal::new(10, 0), c.close);
    assert_eq!(10, c.cost);
}

#[test]
fn test_candle_update() {
    let mut candle = Candle::new();

    let mut trade1 = Trade::new();
    trade1.timestamp = 1;
    trade1.price = Decimal::new(10, 0);
    trade1.amount = 10;
    candle.update(&trade1);

    trade1.timestamp = 2;
    trade1.price = Decimal::new(5, 0);
    trade1.amount = 10;
    candle.update(&trade1);

    trade1.timestamp = 3;
    trade1.price = Decimal::new(7, 0);
    trade1.amount = 10;
    candle.update(&trade1);

    trade1.timestamp = 4;
    trade1.price = Decimal::new(55, 0);
    trade1.amount = 10;
    candle.update(&trade1);

    trade1.timestamp = 5;
    trade1.price = Decimal::new(30, 0);
    trade1.amount = 10;
    candle.update(&trade1).unwrap();

    println!("{:?}", candle);

    assert_eq!(Decimal::new(10, 0), candle.open);
    assert_eq!(Decimal::new(55, 0), candle.high);
    assert_eq!(Decimal::new(5, 0), candle.low);
    assert_eq!(Decimal::new(30, 0), candle.close);

    assert_eq!(50, candle.cost);

    trade1.timestamp = 22;
    trade1.price = Decimal::new(80, 0);
    trade1.amount = 10;
    candle.update(&trade1).unwrap();

    assert_eq!(Decimal::new(80, 0), candle.high);
    assert_eq!(Decimal::new(80, 0), candle.close);
    assert_eq!(60, candle.cost);

    trade1.timestamp = 61;
    trade1.price = Decimal::new(80, 0);
    trade1.amount = 10;

    let mut c = match candle.update(&trade1) {
        Ok(_) => panic!("Should emit Err() with new handle"),
        Err(candle) => candle,
    };

    assert_eq!(Decimal::new(80, 0), c.open);
    assert_eq!(Decimal::new(80, 0), c.high);
    assert_eq!(Decimal::new(80, 0), c.low);
    assert_eq!(Decimal::new(80, 0), c.close);
    assert_eq!(10, c.cost);
}

#[test]
fn test_candle_align_timestamp() {
    assert_eq!(1618934400000, Candle::_align_timestamp(60, 1618936720000));
    assert_eq!(1618938000000, Candle::_align_timestamp(60, 1618938005000));

    assert_eq!(1618929960000, Candle::_align_timestamp(1, 1618929985000));
}
