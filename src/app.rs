use std::collections::HashSet;

use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    config::{
        ACCOUNT_LOOKUP_BUFFER, MARKET_INPUT_BUFFER, POSITION_UPDATE_BUFFER,
        VOLATILITY_WINDOW_DURATION, VOLATILITY_WINDOW_MAX_POINTS,
    },
    hyperliquid::{hl_account_state_scanner, hl_market_data},
    market::{Coin, MarketInput},
    position::{AccountLookupRequest, run_position_tracker},
    price_data::{PricePoint, PriceWindow},
    volatility::{VolatilityDetector, evaluate_volatility},
};

/// Creates each part of the application pipeline and waits for all tasks to
/// finish. The channels connect market data, account lookups and positions.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Market data goes from the WebSocket task to process_market_inputs.
    let (market_tx, market_rx) = mpsc::channel(MARKET_INPUT_BUFFER);

    // New addresses go from process_market_inputs to the REST account scanner.
    let (account_lookup_tx, account_lookup_rx) = mpsc::channel(ACCOUNT_LOOKUP_BUFFER);

    // Filtered results go from the account scanner to the position consumer.
    let (position_update_tx, position_update_rx) = mpsc::channel(POSITION_UPDATE_BUFFER);

    // Receive prices and trades from the Hyperliquid WebSocket.
    let market_data_task = tokio::spawn(hl_market_data(Coin::Btc, market_tx));

    // Consume queued addresses and make the rate-limited REST requests.
    let account_state_task = tokio::spawn(hl_account_state_scanner(
        account_lookup_rx,
        position_update_tx,
    ));

    // Consume the position updates created by the account scanner.
    let position_update_task = tokio::spawn(run_position_tracker(position_update_rx));

    // Process market messages in this task until the market channel closes.
    process_market_inputs(market_rx, account_lookup_tx).await;

    // If the producer panicked or the WebSocket ended unexpectedly, surface
    // that failure instead of silently exiting the application.
    market_data_task.await??;
    account_state_task.await?;
    position_update_task.await?;

    Ok(())
}

/// Consumes the normalized messages produced by hl_market_data. Price messages
/// update volatility state and trade messages discover addresses to scan.
async fn process_market_inputs(
    mut market_rx: Receiver<MarketInput>,
    account_lookup_tx: Sender<AccountLookupRequest>,
) {
    // Keep the recent prices used by the volatility calculation.
    let mut price_window =
        PriceWindow::new(VOLATILITY_WINDOW_DURATION, VOLATILITY_WINDOW_MAX_POINTS);

    // Keep the volatility cooldown state between price messages.
    let mut detector = VolatilityDetector::new();
    // Avoid sending the same address to the account scanner more than once.
    let mut discovered_addresses = HashSet::new();

    // Wait without blocking until the WebSocket task sends the next message.
    while let Some(input) = market_rx.recv().await {
        input.display();

        match input {
            MarketInput::PriceUpdate {
                coin,
                mark_price,
                timestamp,
            } => {
                // Add the mark price to the rolling volatility window.
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
            MarketInput::TradeObserved {
                coin,
                buyer,
                seller,
                ..
            } => {
                // A trade changes both the buyer and seller positions.
                for address in [buyer, seller] {
                    // HashSet::insert returns true only for a new address.
                    if discovered_addresses.insert(address.clone()) {
                        let request = AccountLookupRequest { address, coin };

                        // Queue the address without making the REST call here.
                        account_lookup_tx
                            .send(request)
                            .await
                            .expect("account lookup task should remain active");
                    }
                }
            }
        }
    }
}
