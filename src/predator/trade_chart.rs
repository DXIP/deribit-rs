use rust_decimal::prelude::Decimal;
use std::collections::BTreeMap;

/// TODO: introduce standalone types instead of aliases
///
type Milliseconds = u64;
type Seconds = u64;
type Minutes = u64;

pub enum TickDirection {
    PlusTick = 0,
    ZeroPlusTick = 1,
    MinusTick = 2,
    ZeroMinusTick = 3,
}

pub enum Direction {
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
pub struct Trade {
    pub trade_seq: u64,
    pub trade_id: String,
    pub timestamp: Milliseconds,
    pub tick_direction: TickDirection,
    pub price: Decimal,
    pub mark_price: Decimal,
    pub instrument_name: String,
    pub index_price: Decimal,
    pub direction: Direction,
    pub amount: u64,
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
pub struct Candle {
    pub duration: Seconds,
    pub start_timestamp: Milliseconds,

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

    fn _align_timestamp(r: Resolution, timestamp: Milliseconds) -> Milliseconds {
        (timestamp / 1000) / ((r as u64) * 60) * ((r as u64) * 60) * 1000
    }

    fn create(t: &Trade, r: &Resolution) -> Candle {
        Candle {
            duration: (*r as u64) * 60,
            start_timestamp: Self::_align_timestamp(Resolution::Hour1, t.timestamp),
            open: t.price,
            high: t.price,
            low: t.price,
            close: t.price,
            cost: t.amount,
            volume: 0.0,
        }
    }

    fn update(&mut self, t: &Trade) -> Result<(), Candle> {
        //println!(
        //    "{} + {} < {}",
        //    self.start_timestamp, self.duration, t.timestamp
        //);
        if self.start_timestamp + self.duration < t.timestamp {
            Err(Candle::create(&t, &Resolution::Minute1))
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

pub type TradeCandleChart = BTreeMap<Milliseconds, Candle>;

pub struct TradeLog {
    pub trade_log: Vec<Trade>,

    pub observers: BTreeMap<Resolution, TradeCandleChart>,
}

impl TradeLog {
    pub fn new() -> TradeLog {
        TradeLog {
            trade_log: vec![],
            observers: BTreeMap::new(),
        }
    }
    pub fn trade_chart(&mut self, r: Resolution) -> &TradeCandleChart {
        self.observers.entry(r).or_insert(BTreeMap::new())
    }

    pub fn new_trade(&mut self, t: &Trade) -> () {
        //self.trade_log
        for (r, chart) in self.observers.iter_mut() {
            if let Some(candle) = chart.values_mut().last() {
                let _ = match candle.update(t) {
                    //.expect("new candle case not handled");
                    Err(c) => {
                        //println!("inserting candle");
                        chart.insert(c.start_timestamp, c);
                        ()
                    }
                    _ => (),
                };
            } else {
                //println!("adding candle, empty map");
                chart.insert(t.timestamp, Candle::create(t, r));
            }
        }
    }
}

#[repr(u64)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Resolution {
    Minute1 = 1,
    Minute3 = 3,
    Minute5 = 5,
    Minute10 = 10,
    Minute15 = 15,
    Minute30 = 30,

    Hour1 = 60,
    Hour2 = 120,
    Hour3 = 180,
    Hour4 = 240,
    Hour6 = 360,
    Hour12 = 720,

    Day1 = 1440,
}

#[test]
fn test_candle_create() {
    let mut trade1 = Trade::new();
    trade1.timestamp = 1;
    trade1.price = Decimal::new(10, 0);
    trade1.amount = 10;
    let mut c = Candle::create(&trade1, &Resolution::Minute1);

    assert_eq!(Decimal::new(10, 0), c.open);
    assert_eq!(Decimal::new(10, 0), c.high);
    assert_eq!(Decimal::new(10, 0), c.low);
    assert_eq!(Decimal::new(10, 0), c.close);
    assert_eq!(10, c.cost);

    let mut c = Candle::create(&trade1, &Resolution::Minute3);
    assert_eq!((Resolution::Minute3 as u64) * 60, c.duration);
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
    assert_eq!(
        1618934400000,
        Candle::_align_timestamp(Resolution::Hour1, 1618936720000)
    );
    assert_eq!(
        1618938000000,
        Candle::_align_timestamp(Resolution::Hour1, 1618938005000)
    );

    assert_eq!(
        1618929960000,
        Candle::_align_timestamp(Resolution::Minute1, 1618929985000)
    );
}

#[test]
fn test_trade_log() {
    let trade_log = TradeLog::new();
    assert_eq!(0, trade_log.trade_log.len());
}

#[test]
fn test_trade_log_charts() {
    let mut trade_log = TradeLog::new();
    assert_eq!(0, trade_log.observers.len());

    trade_log.trade_chart(Resolution::Minute1);
    trade_log.trade_chart(Resolution::Minute3);
    trade_log.trade_chart(Resolution::Minute5);
    trade_log.trade_chart(Resolution::Minute1);
    trade_log.trade_chart(Resolution::Minute3);
    trade_log.trade_chart(Resolution::Minute5);

    assert_eq!(3, trade_log.observers.len());
    assert_eq!(true, trade_log.observers.contains_key(&Resolution::Minute1));
    assert_eq!(true, trade_log.observers.contains_key(&Resolution::Minute3));
    assert_eq!(true, trade_log.observers.contains_key(&Resolution::Minute5));
    assert_eq!(
        false,
        trade_log.observers.contains_key(&Resolution::Minute15)
    );
    assert_eq!(
        false,
        trade_log.observers.contains_key(&Resolution::Minute30)
    );
}

#[test]
fn test_trade_log_new_trade() {
    let mut trade_log = TradeLog::new();

    let mut trade1 = Trade::new();
    trade1.timestamp = 1;
    trade1.price = Decimal::new(10, 0);
    trade1.amount = 10;

    println!("new_trade1");
    trade_log.new_trade(&trade1);
    assert_eq!(0, trade_log.observers.len());

    trade_log.trade_chart(Resolution::Minute1);
    trade_log.trade_chart(Resolution::Minute3);
    trade_log.trade_chart(Resolution::Minute5);
    assert_eq!(3, trade_log.observers.len());

    println!("new_trade2");
    trade_log.new_trade(&trade1);
    assert_eq!(
        1,
        trade_log.observers.get(&Resolution::Minute1).unwrap().len()
    );
    assert_eq!(
        1,
        trade_log.observers.get(&Resolution::Minute3).unwrap().len()
    );
    assert_eq!(
        1,
        trade_log.observers.get(&Resolution::Minute5).unwrap().len()
    );

    let minute1_candle = trade_log
        .observers
        .get(&Resolution::Minute1)
        .unwrap()
        .values()
        .last()
        .unwrap();

    assert_eq!(Decimal::new(10, 0), minute1_candle.open);
    assert_eq!(Decimal::new(10, 0), minute1_candle.high);
    assert_eq!(Decimal::new(10, 0), minute1_candle.low);
    assert_eq!(Decimal::new(10, 0), minute1_candle.close);

    trade1.timestamp = 100;
    trade1.price = Decimal::new(70, 0);
    trade1.amount = 10;

    println!("new_trade3");
    trade_log.new_trade(&trade1);

    println!("{:?}", trade_log.observers);
    assert_eq!(
        2,
        trade_log.observers.get(&Resolution::Minute1).unwrap().len()
    );
    assert_eq!(
        1,
        trade_log.observers.get(&Resolution::Minute3).unwrap().len()
    );
    assert_eq!(
        1,
        trade_log.observers.get(&Resolution::Minute5).unwrap().len()
    );
}
