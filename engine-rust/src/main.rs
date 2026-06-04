use std::error::Error;

use crate::market_data::{
    clients::run_client::run_market_data_clients, config::MarketDataConfig,
    runtime::MarketDataRuntime, startup,
};
mod log;
mod market_data;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _guard = log::init_logging();
    tracing::info!("Market data runtime starting");

    let market_data_config = MarketDataConfig::default();
    let candle_keys = market_data_config.candle_keys.clone();
    let mut runtime = MarketDataRuntime::new(market_data_config);
    
    tracing::info!(streams = candle_keys.len(), candle_keys = ?candle_keys, "Candle streams configured");

    tracing::info!("Starting engine...");
    startup::prepare_market_data_runtime(&mut runtime, &candle_keys).await?;


    tracing::info!("Starting live market data stream");
    // same keys as REST seed — client rebuilds subs on each connect
    run_market_data_clients(&candle_keys, &mut runtime).await?;

    Ok(())
}
