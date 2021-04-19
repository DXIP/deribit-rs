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

    let req = PublicSubscribeRequest::new(&["book.BTC-PERPETUAL.100ms".into()]);

    let _ = client.call(req).await?.await?;

    while let Some(m) = subscription.next().await {
        if let Ok(val) = m {
            if let SubscriptionParams::Subscription(p) = val.params {
                match p {
                    SubscriptionData::Book(book) => {
                        let book_data = book.data;
                        order_book.load(&book_data);
                        println!("{:?}", order_book);
                        println!("spread: {}", order_book.spread);
                    }
                    _ => (),
                }
                //println!("{:?}", book.data);
            }

            panic! {"done"};
        }
    }
}
