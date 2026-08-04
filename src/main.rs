use futures::StreamExt;
use hypersdk::hypercore::{self, Incoming, types::Subscription, ws::Event};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::init_with_level(log::Level::Info);

    let client = hypercore::mainnet();
    let mut ws = client.websocket();

    // subscribing to bct trades
    ws.subscribe(Subscription::Trades {
        coin: "BTC".to_string(),
    });

    log::info!("Subscribed to BTC 1m candles. Waiting for updates...\n");

    while let Some(event) = ws.next().await
    // awais for messsage from the stream
    {
        match event {
            Event::Connected => {
                log::info!("Websocket Connected!");
            }
            Event::Disconnected => {
                log::info!("Websocket Disconnected")
            }
            Event::Message(Incoming::Trades(trades)) => 
            {
                for trade in trades 
                {
                    log::info!("Received trade: {:?}", trade);
                }
            }
             Event::Message(_) => {} // for all rest of the messages we don't care about
        }
    }

    Ok(())
}
