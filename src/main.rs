mod hyperliquid;
mod market;
mod price_data;

use std::time::Duration;

use hyperliquid::hl_market_data;
use market::{Coin, MarketInput};
use price_data::{PricePoint, PriceWindow};
use tokio::sync::mpsc;

const MARKET_INPUT_BUFFER: usize = 256;
const VOLATILITY_WINDOW_SECONDS: u64 = 60;
const VOLATILITY_WINDOW_MAX_POINTS: usize = 1_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(MARKET_INPUT_BUFFER);
    let mut price_window = PriceWindow::new(
        Duration::from_secs(VOLATILITY_WINDOW_SECONDS),
        VOLATILITY_WINDOW_MAX_POINTS,
    );

    let market_data_task = tokio::spawn(hl_market_data(Coin::Btc, tx));

    while let Some(input) = rx.recv().await {
        input.display();

        match input {
            MarketInput::PriceUpdate {
                mark_price,
                timestamp,
                ..
            } => {
                price_window.push(PricePoint::new(mark_price, timestamp));

                if let Some(change) = price_window.percentage_change() {
                    println!("Change inside rolling window: {change}%");
                }
            }
        }
    }

    // If the producer panicked or the WebSocket ended unexpectedly, surface
    // that failure instead of silently exiting the application.
    market_data_task.await??;

    Ok(())
}
