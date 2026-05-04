use crate::market_data::{engine::Engine, hyperliquid::{hl_client::run_hyperliquid_client, protocols::subscribe::{ subscribe_candle,}}, 
            types::candle::{Coins, Interval}};
mod market_data;
mod log;

#[tokio::main]
async fn main()->Result<(), Box<dyn std::error::Error>> 
{
    let _guard = log::init_logging();
    tracing::info!("Engine starting");

    let mut engine = Engine::new();

    // Stream to subscribe
    let subs = vec!
    [
        subscribe_candle(Coins::HYPE, Interval::M5),
        subscribe_candle(Coins::HYPE, Interval::M15),
        subscribe_candle(Coins::HYPE, Interval::H1),
    ];
    run_hyperliquid_client(subs, &mut engine).await?;

    Ok(())
}

