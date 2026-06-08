use std::error::Error;

use tokio::sync::mpsc;

use crate::{
    market_data::{
        alert_subscriptions::command::SubscriptionManager, config::MarketDataConfig,
        constans::BUFFER_SIZE_FOR_MPSC, coordinator::run_live, runtime::MarketDataRuntime, startup,
    },
    subscription_stream::{
        redis::RedisSubscriptionStream,
        subscription_constants::{REDIS_ADDRESS, SUBSCRIPTION_CHANNEL},
    },
};

mod log;
mod market_data;
mod subscription_stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // logging to file + console for the whole process
    let _guard = log::init_logging();
    tracing::info!("Market data runtime starting");

    // default config — candle keys (HYPE m5/m15/h1), buffer sizes, atr defaults
    let market_data_config = MarketDataConfig::default();
    let candle_keys = market_data_config.candle_keys.clone();

    // composition root — candle store, alert books, evaluators all live here
    let mut runtime = MarketDataRuntime::new(market_data_config);

    tracing::info!(
        streams = candle_keys.len(),
        candle_keys = ?candle_keys,
        "Configurations set successefully"
    );

    // REST seed candles only — subscriptions come from redis after startup
    tracing::info!("Starting engine...");
    startup::prepare_market_data_runtime(&mut runtime, &candle_keys).await?;

    // mpsc channel — redis task sends SubscriptionManager, run_live receives in select!
    let (subscription_sender, subscription_receiver) =
        mpsc::channel::<SubscriptionManager>(BUFFER_SIZE_FOR_MPSC);

    // background task: SUBSCRIBE alert_subscriptions, parse JSON, forward on sender
    let subscription_stream = RedisSubscriptionStream::new(REDIS_ADDRESS, subscription_sender)?;

    tokio::spawn(async move {
        if let Err(err) = subscription_stream
            .bind_to_stream(SUBSCRIPTION_CHANNEL)
            .await
        {
            tracing::error!(error = %err, "redis subscription listener stopped");
        }
    });

    // main loop — hyperliquid WS candles + health + subscription recv → process()
    tracing::info!("Starting live market data stream");
    run_live(&mut runtime, &candle_keys, subscription_receiver).await?;

    Ok(())
}
