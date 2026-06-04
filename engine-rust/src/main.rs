use std::error::Error;

use tokio::sync::mpsc;

use crate::market_data::{
    alert_subscriptions::command::SubscriptionManager,
    config::MarketDataConfig,
    constans::BUFFER_SIZE_FOR_MPSC,
    coordinator::run_live,
    runtime::MarketDataRuntime,
    startup,
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

    tracing::info!(
        streams = candle_keys.len(),
        candle_keys = ?candle_keys,
        "Configurations set successefully"
    );

    tracing::info!("Starting engine...");
    startup::prepare_market_data_runtime(&mut runtime, &candle_keys).await?;

    let (subscription_sender, subscription_receiver) =
        mpsc::channel::<SubscriptionManager>(BUFFER_SIZE_FOR_MPSC);
    let _subscription_sender = subscription_sender; // stream task will own this next

    tracing::info!("Starting live market data stream");
    run_live(&mut runtime, &candle_keys, subscription_receiver).await?;

    Ok(())
}
