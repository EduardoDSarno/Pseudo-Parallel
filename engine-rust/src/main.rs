use std::error::Error;

use crate::market_data::{
    clients::hyperliquid::hl_client::run_hyperliquid_client,
    config::MarketDataConfig,
    runtime::MarketDataRuntime,
    startup::seed_engine_from_rest,
    subscriptions::placeholder::dev_signal_subscriptions,
    types::{CandleKey, Coins, Interval},
};
mod log;
mod market_data;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _guard = log::init_logging();
    tracing::info!("Market data runtime starting");

    let market_data_config = MarketDataConfig::default();
    let mut runtime = MarketDataRuntime::new(market_data_config);
    tracing::info!(
        max_closed_candles = market_data_config.max_closed_candles,
        "Market data runtime initialized"
    );

    // Candle streams we want to seed first and then keep receiving live data from.
    let candle_keys = vec![
        CandleKey::new(Coins::HYPE, Interval::M5),
        CandleKey::new(Coins::HYPE, Interval::M15),
        CandleKey::new(Coins::HYPE, Interval::H1),
    ];
    tracing::info!(streams = candle_keys.len(), candle_keys = ?candle_keys, "Candle streams configured");

    tracing::info!("Starting REST seed");
    seed_engine_from_rest(&mut runtime, &candle_keys).await?;
    tracing::info!("REST seed finished");

    let subscriptions = dev_signal_subscriptions();
    tracing::info!(
        subscription_count = subscriptions.len(),
        "Loading dev signal subscriptions"
    );
    runtime.load_signal_subscriptions(subscriptions)?;

    tracing::info!("Starting live market data stream");
    // same keys as REST seed — client rebuilds subs on each connect
    run_hyperliquid_client(&candle_keys, &mut runtime).await?;

    Ok(())
}
