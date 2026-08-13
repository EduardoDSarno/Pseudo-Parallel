mod hyperliquid;
mod market;

use hyperliquid::hl_market_data;
use market::Coin;
use tokio::sync::mpsc;

const MARKET_INPUT_BUFFER: usize = 256;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(MARKET_INPUT_BUFFER);

    let market_data_task = tokio::spawn(hl_market_data(Coin::Btc, tx));

    while let Some(input) = rx.recv().await {
        input.display();
    }

    // If the producer panicked or the WebSocket ended unexpectedly, surface
    // that failure instead of silently exiting the application.
    market_data_task.await??;

    Ok(())
}
