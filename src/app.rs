use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    watch,
};
use tokio::time::Instant;

use crate::{
    account_refresh_scheduler::AccountRefreshScheduler,
    accounts::{AccountLookupRequest, AddressRefreshRegistry},
    coin::Coin,
    config::{
        ACCOUNT_LOOKUP_BUFFER, MARKET_INPUT_BUFFER, POSITION_UPDATE_BUFFER,
        VOLATILITY_WINDOW_DURATION, VOLATILITY_WINDOW_MAX_POINTS,
    },
    helper::send_account_lookup_request,
    hyperliquid::{hl_account_state_scanner, hl_market_data},
    market::MarketInput,
    market_processor::process_market_input,
    position::run_position_tracker,
    price_data::{CurrentPrice, PriceWindow},
    volatility::VolatilityDetector,
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

    // Unlike mpsc, watch keeps only the latest market price. The initial value
    // is None because positions may arrive before the first price message.
    let (current_price_tx, current_price_rx) = watch::channel::<Option<CurrentPrice>>(None);

    // Receive prices and trades from the Hyperliquid WebSocket.
    let market_data_task = tokio::spawn(hl_market_data(Coin::Btc, market_tx));

    // Consume queued addresses and make the rate-limited REST requests.
    let account_state_task = tokio::spawn(hl_account_state_scanner(
        account_lookup_rx,
        position_update_tx,
    ));

    // Consume the position updates created by the account scanner.
    let position_update_task =
        tokio::spawn(run_position_tracker(position_update_rx, current_price_rx));

    // Process market messages in this task until the market channel closes.
    process_market_inputs(market_rx, account_lookup_tx, current_price_tx).await;

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
    current_price_tx: watch::Sender<Option<CurrentPrice>>,
) 
{
    // Keep the recent prices used by the volatility calculation.
    let mut price_window =
        PriceWindow::new(VOLATILITY_WINDOW_DURATION, VOLATILITY_WINDOW_MAX_POINTS);

    // Keep the volatility cooldown state between price messages.
    let mut detector = VolatilityDetector::new();
    // Remember when each typed address was last queued for an account lookup.
    let mut address_refreshes = AddressRefreshRegistry::new();
    // Keep future account refreshes ordered by their execution deadline.
    let mut refresh_scheduler = AccountRefreshScheduler::new();

    loop
    {
        let next_refresh_at = refresh_scheduler.next_deadline();

        // Wait for either market data or the earliest scheduled refresh.
        tokio::select! {
            // Market Data
            maybe_input = market_rx.recv() =>
            {
                // Check for Some valid market Data
                let Some(input) = maybe_input else
                {
                    log::info!("Invalid Input received (None)");
                    break;
                };

                // Display event in text (console or log)
                input.display();

                process_market_input(
                    input,
                    &mut price_window,
                    &mut detector,
                    &current_price_tx,
                    &mut address_refreshes,
                    &mut refresh_scheduler,
                    &account_lookup_tx,
                )
                .await;
            }

            _ = async {
                AccountRefreshScheduler::wait_until(next_refresh_at).await;
            } =>
            {
                let now = Instant::now();
                let scheduled_requests = refresh_scheduler.take_due(now);

                for request in scheduled_requests {
                    if address_refreshes.take_due_refresh(&request.address) {
                        send_account_lookup_request(&account_lookup_tx, request).await;
                    }
                }
            }
        }
    }
}
