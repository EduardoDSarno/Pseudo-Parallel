mod hyperliquid;
mod market;
mod price_data;
mod volatility;

use std::{collections::HashSet, time::Duration};

use hyperliquid::{hl_account_state_scanner, hl_market_data};
use market::{Coin, MarketInput};
use price_data::{PricePoint, PriceWindow};
use tokio::sync::mpsc;
use volatility::*;

const MARKET_INPUT_BUFFER: usize = 256;
const ADDRESS_QUEUE_BUFFER: usize = 2_048;
const VOLATILITY_WINDOW_SECONDS: u64 = 60;
const VOLATILITY_WINDOW_MAX_POINTS: usize = 1_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::init_with_level(log::Level::Info);

    let (tx, mut rx) = mpsc::channel(MARKET_INPUT_BUFFER);
    let (address_tx, address_rx) = mpsc::channel::<String>(ADDRESS_QUEUE_BUFFER);
    let account_state_task = tokio::spawn(hl_account_state_scanner(address_rx));

    let mut price_window = PriceWindow::new(
        Duration::from_secs(VOLATILITY_WINDOW_SECONDS),
        VOLATILITY_WINDOW_MAX_POINTS,
    );

    let market_data_task = tokio::spawn(hl_market_data(Coin::Btc, tx));

    let mut detector = VolatilityDetector::new();
    // for addresses already queued
    let mut discovered_addresses = HashSet::new();

    while let Some(input) = rx.recv().await {
        input.display();

        match input {
            MarketInput::PriceUpdate {
                coin,
                mark_price,
                timestamp,
            } => {
                price_window.push(PricePoint::new(mark_price, timestamp));

                // Check the latest rolling-window movement for a spike.
                if let Some(change) = price_window.percentage_change() {
                    if let Some(spike) = evaluate_volatility(coin, change, timestamp, &mut detector)
                    {
                        spike.display();
                    }

                    println!("Change inside rolling window: {change}%");
                }
            }
            MarketInput::TradeObserved { buyer, seller, .. } => {
                // The array lets us apply the same discovery logic to both participants.
                for address in [buyer, seller] {
                    // HashSet::insert returns true only for a newly discovered address.
                    if discovered_addresses.insert(address.clone()) {
                        address_tx
                            .send(address)
                            .await
                            .expect("address queue task should remain active");
                    }
                }
            }
        }
    }

    // If the producer panicked or the WebSocket ended unexpectedly, surface
    // that failure instead of silently exiting the application.
    market_data_task.await??;

    // Close the queue after market data ends, then let its consumer finish.
    drop(address_tx);
    account_state_task.await?;

    Ok(())
}
