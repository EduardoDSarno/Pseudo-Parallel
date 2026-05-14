use std::error::Error;

use crate::{market_data::{engine::Engine, hyperliquid::{hl_client::run_hyperliquid_client, protocols::subscribe::subscribe_candle}, types::{CandleKey, Coins, Interval}}, startup::seed_engine_from_rest};
mod market_data;
mod log;
mod startup;

#[tokio::main]
async fn main()->Result<(), Box<dyn Error>> 
{
    let _guard = log::init_logging();
    tracing::info!("Engine starting");

    let mut engine = Engine::new();
    tracing::info!("Market data engine initialized");

    // Candle streams we want to seed first and then keep receiving live data from.
    let candle_keys = vec!
    [
        CandleKey::new(Coins::HYPE, Interval::M5),
        CandleKey::new(Coins::HYPE, Interval::M15),
        CandleKey::new(Coins::HYPE, Interval::H1),
    ];
    tracing::info!(streams = candle_keys.len(), candle_keys = ?candle_keys, "Candle streams configured");

    tracing::info!("Starting REST seed");
    seed_engine_from_rest(&mut engine, &candle_keys).await?;
    tracing::info!("REST seed finished");

    // Create the WebSocket subscriptions from the same keys we already seeded.
    let subs = candle_keys
        .into_iter()
        .map(|key| subscribe_candle(key.coin, key.interval))
        .collect();
    tracing::info!("WebSocket subscriptions created");

    tracing::info!("Starting live market data stream");
    run_hyperliquid_client(subs, &mut engine).await?;
    tracing::warn!("Live market data stream stopped");

    Ok(())
}
