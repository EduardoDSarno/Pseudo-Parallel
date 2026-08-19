use tokio::sync::{mpsc::Sender, watch};

use crate::{
    account_refresh_scheduler::AccountRefreshScheduler,
    accounts::{AccountLookupRequest, AddressRefreshAction, AddressRefreshRegistry},
    helper::send_account_lookup_request,
    market::MarketInput,
    price_data::{CurrentPrice, PricePoint, PriceWindow},
    volatility::{VolatilityDetector, evaluate_volatility},
};

/// Applies one normalized market event to the price or account-refresh state.
pub async fn process_market_input(
    input: MarketInput,
    price_window: &mut PriceWindow,
    detector: &mut VolatilityDetector,
    current_price_tx: &watch::Sender<Option<CurrentPrice>>,
    address_refreshes: &mut AddressRefreshRegistry,
    refresh_scheduler: &mut AccountRefreshScheduler,
    account_lookup_tx: &Sender<AccountLookupRequest>,
) {
    match input {
        MarketInput::PriceUpdate {
            coin,
            mark_price,
            timestamp,
        } => {
            // Add the mark price to the rolling volatility window.
            price_window.push(PricePoint::new(mark_price, timestamp));

            // Publish only the latest price to the heatmap tracker. A slow
            // consumer does not need to process stale prices first.
            current_price_tx.send_replace(Some(CurrentPrice {
                coin,
                mark_price,
                observed_at: timestamp,
            }));

            // Check the latest rolling-window movement for a spike.
            if let Some(change) = price_window.percentage_change() {
                if let Some(spike) = evaluate_volatility(coin, change, timestamp, detector) {
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
                let refresh_action = address_refreshes.register_activity(address);
                let request = AccountLookupRequest { address, coin };

                match refresh_action {
                    // When an account is ready to be requested.
                    AddressRefreshAction::RequestNow => {
                        send_account_lookup_request(account_lookup_tx, request).await;
                    }
                    // When cooldown is active, add it to the scheduled refreshes.
                    AddressRefreshAction::ScheduleAt(deadline) => {
                        refresh_scheduler.schedule(deadline, request);
                    }
                    AddressRefreshAction::Nothing => {}
                }
            }
        }
    }
}
