use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use hypersdk::hypercore;
use hypersdk::hypercore::{types::*, ws::Event};
use tokio::sync::mpsc::Sender;

use crate::market::{Coin, MarketInput};

/// Subscribes to the active asset context for the given coin and streams
/// every price update to `tx` for as long as the connection stays alive.
/// The underlying connection auto-reconnects and re-subscribes on its own,
/// so this only returns once the receiving end is dropped.
pub async fn hl_market_data(coin: Coin, tx: Sender<MarketInput>) -> Result<(), std::io::Error> {
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
                if tx.send(input).await.is_err() {
                    return Ok(());
                }
            }
            Event::Message(Incoming::Trades(trades)) => {
                for trade in trades {
                    let trade_timestamp = UNIX_EPOCH + Duration::from_millis(trade.time);
                    let Some(input) = MarketInput::create_trade_record(
                        coin,
                        trade.users[0],
                        trade.users[1],
                        trade_timestamp,
                    ) else {
                        continue;
                    };

                    // clean shut down when the receiving end has been dropped
                    if tx.send(input).await.is_err() {
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
