mod hyperliquid;
mod market;

use hyperliquid::hl_market_data;
use market::MarketInput;
use tokio::sync::mpsc;

const MARKET_INPUT_BUFFER: usize = 256;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(MARKET_INPUT_BUFFER);

    let market_data_task = tokio::spawn(hl_market_data(tx));

    while let Some(input) = rx.recv().await {
        match input {
            MarketInput::PriceUpdate { coin, mark_price } => {
                println!("{coin} mark price: {mark_price}");
            }
        }
    }

    // If the producer panicked or the WebSocket ended unexpectedly, surface
    // that failure instead of silently exiting the application.
    market_data_task.await??;

    Ok(())
}
