use rust_decimal::prelude::Decimal;

struct TradeLog {
    pub trade_log: Vec<()>,
}

struct TradeChart<T: Resolution> {
    pub candles: Vec<Candle<T>>,
}

#[derive(Debug)]
struct Candle<T: Resolution> {
    pub resolution: T,
    pub start_timestamp: u64,

    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: f64,
}

impl<T: Resolution> Candle<T> {
    pub fn new(r: T) -> Candle<T> {
        Candle {
            resolution: r,
            start_timestamp: 0,
            open: Decimal::new(0, 2),
            high: Decimal::new(0, 2),
            low: Decimal::new(0, 2),
            close: Decimal::new(0, 2),
            volume: 0.0,
        }
    }
}

trait Resolution {
    fn duration() -> u8 {
        0
    }
}

#[derive(Debug)]
struct ResolutionMinute1 {}
impl Resolution for ResolutionMinute1 {
    fn duration() -> u8 {
        1
    }
}

#[derive(Debug)]
struct ResolutionMinute3 {}
impl Resolution for ResolutionMinute3 {
    fn duration() -> u8 {
        3
    }
}

#[derive(Debug)]
struct ResolutionMinute5 {}
impl Resolution for ResolutionMinute5 {
    fn duration() -> u8 {
        5
    }
}
/*
enum r {
    MINUTE_1 = 1,
    MINUTE_3 = 3,
    MINUTE_5 = 5,
    MINUTE_10 = 10,
    MINUTE_15 = 15,
    MINUTE_30 = 30,
    HOUR_1 = 60,
    HOUR_2 = 120,
    HOUR_3 = 180,
    HOUR_4 = 240,
    HOUR_6 = 360,
    HOUR_12 = 720,
    DAY_1 = 1440,
}*/

#[test]
fn test_candle() {
    println!("starting test");
    let c = Candle {
        resolution: ResolutionMinute1 {},
        start_timestamp: 0,
        open: Decimal::new(0, 2),
        high: Decimal::new(0, 2),
        low: Decimal::new(0, 2),
        close: Decimal::new(0, 2),
        volume: 0.0,
    };
    println!("local candle {:?}", c);

    let c1 = Candle::new(ResolutionMinute5 {});
    println!("Candle::new {:?}", c1);
    assert_eq!(0, 1);
}
