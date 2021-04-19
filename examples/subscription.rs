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

#[throws(Error)]
#[tokio::main]
async fn main() {
    let _ = dotenv();
    init();

    let mut order_book = predator::OrderBook::new();

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
                    }

                    _ => (),
                }
                //println!("{:?}", book.data);
            }

            //panic! {"done"};
        }
    }
}
