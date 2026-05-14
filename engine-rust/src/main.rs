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

    // Candle streams we want to seed first and then keep receiving live data from.
    let candle_keys = vec!
    [
        CandleKey::new(Coins::HYPE, Interval::M5),
        CandleKey::new(Coins::HYPE, Interval::M15),
        CandleKey::new(Coins::HYPE, Interval::H1),
    ];

    seed_engine_from_rest(&mut engine, &candle_keys).await?;

    // Create the WebSocket subscriptions from the same keys we already seeded.
    let subs = candle_keys
        .into_iter()
        .map(|key| subscribe_candle(key.coin, key.interval))
        .collect();

    run_hyperliquid_client(subs, &mut engine).await?;

    Ok(())
}
