use deribit::models::{
    HeartbeatType, PublicSubscribeRequest, SetHeartbeatRequest, SubscriptionData,
    SubscriptionParams, TestRequest,
};

use deribit::predator;
use deribit::DeribitBuilder;
use dotenv::dotenv;
use env_logger::init;
use failure::Error;
use fehler::throws;
use futures::StreamExt;
use rust_decimal::prelude::Decimal;

#[throws(Error)]
#[tokio::main]
async fn main() {
    let _ = dotenv();
    init();

    let r = predator::Resolution::Minute1;

    let mut order_book = predator::OrderBook::new();

    let mut trade_log = predator::TradeLog::new();
    trade_log.trade_chart(r);

    let drb = DeribitBuilder::default()
        .subscription_buffer_size(100000usize)
        .build()
        .unwrap();

    let (mut client, mut subscription) = drb.connect().await?;

    let req = PublicSubscribeRequest::new(&[
        "book.ETH-PERPETUAL.100ms".into(),
        "trades.ETH-PERPETUAL.100ms".into(),
    ]);

    let _ = client.call(req).await?.await?;

    while let Some(m) = subscription.next().await {
        if let Ok(val) = m {
            if let SubscriptionParams::Subscription(p) = val.params {
                match p {
                    SubscriptionData::Book(book) => {
                        let book_data = book.data;
                        order_book.update(&book_data);
                        //println!("{:?}", order_book);
                        println!(
                            "lowest_ask: {}\task_size: {}",
                            order_book.ask,
                            order_book.asks.len()
                        );
                        println!(
                            "highest_bid: {}\tbid_size: {}",
                            order_book.bid,
                            order_book.bids.len()
                        );
                        println!("spread: {}", order_book.spread);
                    }

                    SubscriptionData::Trades(trades) => {
                        println!("trades: {:?}", trades.data);
                        for t in trades.data.iter() {
                            let trade = predator::Trade {
                                trade_seq: t.trade_seq,
                                trade_id: t.trade_id.to_string(),
                                timestamp: t.timestamp,
                                tick_direction: predator::TickDirection::PlusTick,
                                price: Decimal::new((t.price * 100.0).round() as i64, 2),
                                mark_price: Decimal::new(0, 0), //Decimal::new((t.mark_price * 100.0).round(), 2),
                                instrument_name: t.instrument_name.to_string(),
                                index_price: Decimal::new(
                                    (t.index_price * 100.0).round() as i64,
                                    2,
                                ),
                                direction: predator::Direction::Sell,
                                amount: t.amount as u64,
                            };

                            trade_log.new_trade(&trade);
                            let chart = trade_log.trade_chart(r);
                            println!("Candles: {:?}", chart);
                        }
                    }

                    _ => (),
                }
                //println!("{:?}", book.data);
            }

            //panic! {"done"};
        }
    }
}
