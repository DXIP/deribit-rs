use deribit::models::{
    HeartbeatType, PublicSubscribeRequest, SetHeartbeatRequest, SubscriptionParams, TestRequest,
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

    let order_book = predator::OrderBook::new();

    let drb = DeribitBuilder::default()
        .subscription_buffer_size(100000usize)
        .build()
        .unwrap();

    let (mut client, mut subscription) = drb.connect().await?;

    let req = PublicSubscribeRequest::new(&["book.BTC-PERPETUAL.100ms".into()]);

    let _ = client.call(req).await?.await?;

    while let Some(m) = subscription.next().await {
        if let Ok(val) = m {
            //println!("{:?}", val);
            order_book.load(val.params.data);
            println!("{:?}", order_book);

            panic!{"done"};
        }
    }
}
