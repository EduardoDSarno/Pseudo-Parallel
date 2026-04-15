use futures::StreamExt;
use hypersdk::hypercore::{self, types::*, ws::Event};



async fn websocket() -> Result<(), Box<dyn std::error::Error>>
{
    let mut ws = hypercore::mainnet_ws();

    // Subscribe to market data
    ws.subscribe(Subscription::Trades { coin: "BTC".into() });
    ws.subscribe(Subscription::L2Book { coin: "ETH".into() });


    while let Some(event) = ws.next().await 
    {
        let Event::Message(msg) = event else { continue };
        match msg {
            Incoming::Trades(trades) => 
            {
                for trade in trades {
                    println!("trade {} @ {} size {}", trade.side, trade.px, trade.sz);
                }
            }
            Incoming::L2Book(book) => {
                println!("book update for {}", book.coin);
            }
            _ => {}
        }
    }

   Ok(())
}

