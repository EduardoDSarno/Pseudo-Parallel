use std::time::SystemTime;

use futures::StreamExt;
use hypersdk::hypercore;
use hypersdk::hypercore::{types::*, ws::Event};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::MissedTickBehavior,
};

use crate::{
    accounts::AccountLookupRequest,
    coin::Coin,
    config::{
        CLEARINGHOUSE_REQUEST_INTERVAL, MINIMUM_WHALE_POSITION_USD, hyperliquid_time_to_system_time,
    },
    market::MarketInput,
    position::{PositionUpdate, filter_whale_position},
};

/// Subscribes to the active asset context for the given coin and streams
/// every price update to `tx` for as long as the connection stays alive.
/// The underlying connection auto-reconnects and re-subscribes on its own,
/// so this only returns once the receiving end is dropped.
pub async fn hl_market_data(
    coin: Coin,
    market_tx: Sender<MarketInput>,
) -> Result<(), std::io::Error> {
    let mut ws = hypercore::mainnet_ws();

    // Subscribe to mark-price updates for the selected coin.
    ws.subscribe(Subscription::ActiveAssetCtx {
        coin: coin.as_hyperliquid_symbol().to_owned(),
    });

    // Use the same connection to observe trades for the selected coin.
    ws.subscribe(Subscription::Trades {
        coin: coin.as_hyperliquid_symbol().to_owned(),
    });

    while let Some(event) = ws.next().await {
        match event {
            Event::Connected => {
                println!("Connected to Hyperliquid");
            }
            Event::Disconnected => {
                println!("Disconnected from Hyperliquid");
            }
            Event::Message(Incoming::ActiveAssetCtx { ctx, .. }) => {
                let Some(input) =
                    MarketInput::create_mark_price_update(coin, ctx.mark_px, SystemTime::now())
                else {
                    continue;
                };

                // clean shut down when the receiving end has been dropped
                if market_tx.send(input).await.is_err() {
                    return Ok(());
                }
            }
            Event::Message(Incoming::Trades(trades)) => {
                for trade in trades {
                    let Some(trade_timestamp) = hyperliquid_time_to_system_time(trade.time) else {
                        log::warn!("Ignoring trade with unsupported timestamp: {}", trade.time);
                        continue;
                    };
                    let Some(input) = MarketInput::create_trade_record(
                        coin,
                        trade.users[0],
                        trade.users[1],
                        trade_timestamp,
                    ) else {
                        continue;
                    };

                    // clean shut down when the receiving end has been dropped
                    if market_tx.send(input).await.is_err() {
                        return Ok(());
                    }
                }
            }
            _ => {
                println!("Event: {:?}", event);
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        "Hyperliquid WebSocket stream ended",
    ))
}

/// Consumes discovered addresses and requests their clearinghouse states at a
/// controlled rate. A single reusable client and interval keep the REST API
/// work outside the market-data consumer.
pub async fn hl_account_state_scanner(
    mut account_lookup_rx: Receiver<AccountLookupRequest>,
    position_update_tx: Sender<PositionUpdate>,
) {
    let client = hypercore::mainnet();
    let mut request_interval = tokio::time::interval(CLEARINGHOUSE_REQUEST_INTERVAL);
    request_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    while let Some(request) = account_lookup_rx.recv().await {
        request_interval.tick().await;

        let AccountLookupRequest { address, coin } = request;

        match client.clearinghouse_state(address, None).await {
            Ok(state) => {
                let position =
                    filter_whale_position(address, &state, coin, MINIMUM_WHALE_POSITION_USD);

                let update = PositionUpdate { address, position };
                if position_update_tx.send(update).await.is_err() {
                    return;
                }
            }
            Err(error) => {
                log::warn!("Could not load clearinghouse state for {address}: {error}");
            }
        }
    }
}
