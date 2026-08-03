use std::error::Error;

use tokio::sync::mpsc;

use crate::{
    market_data::{
        alert_subscriptions::command::SubscriptionManager, config::MarketDataConfig,
        constans::BUFFER_SIZE_FOR_MPSC, coordinator::run_live, runtime::MarketDataRuntime,
    },
    redis_transport::{
        alert_publisher::spawn_alert_publisher,
        constants::{ALERTS_FIRED_CHANNEL, REDIS_ADDRESS, SUBSCRIPTION_CHANNEL},
        subscription_listener::RedisSubscriptionListener,
    },
};

mod log;
mod market_data;
mod redis_transport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // logging to file + console for the whole process
    let _guard = log::init_logging();
    tracing::info!("Market data runtime starting");

    // one M5 stream is the live price source
    let market_data_config = MarketDataConfig::default();
    let candle_keys = market_data_config.candle_keys.clone();

    // composition root — candle store, alert book, and publisher live here
    let mut runtime = MarketDataRuntime::new(market_data_config);

    tracing::info!(
        streams = candle_keys.len(),
        candle_keys = ?candle_keys,
        "Configurations set successefully"
    );

    // mpsc channel — redis task sends SubscriptionManager, run_live receives in select!
    let (subscription_sender, subscription_receiver) =
        mpsc::channel::<SubscriptionManager>(BUFFER_SIZE_FOR_MPSC);

    // background task: SUBSCRIBE alert_subscriptions, parse JSON, forward on sender
    let subscription_listener = RedisSubscriptionListener::new(REDIS_ADDRESS, subscription_sender)?;

    tokio::spawn(async move {
        if let Err(err) = subscription_listener
            .bind_to_stream(SUBSCRIPTION_CHANNEL)
            .await
        {
            tracing::error!(error = %err, "redis subscription listener stopped");
        }
    });

    // background task: PUBLISH fired alerts to alerts_fired for the TS backend to consume
    let alert_publisher = spawn_alert_publisher(REDIS_ADDRESS, ALERTS_FIRED_CHANNEL).await?;
    runtime.set_alert_publisher(alert_publisher);

    // main loop — hyperliquid WS candles + health + subscription recv → process()
    tracing::info!("Starting live market data stream");
    run_live(&mut runtime, &candle_keys, subscription_receiver).await?;

    Ok(())
}
