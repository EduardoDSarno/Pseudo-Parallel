use crate::market_data::{engine::Engine, hyperliquid::{hl_client::run_hyperliquid_client, protocols::subscribe::{ subscribe_candle,}}, 
            types::candle::{COINS, Interval}};
mod market_data;



#[tokio::main]
async fn main()->Result<(), Box<dyn std::error::Error>> 
{

    let mut engine = Engine::new();

    // Stream to subscribe
    let subs = vec!
    [
        subscribe_candle(COINS::HYPE, Interval::M5),
        subscribe_candle(COINS::HYPE, Interval::M15),
        subscribe_candle(COINS::HYPE, Interval::H1),
    ];
    run_hyperliquid_client(subs, &mut engine).await?;

    Ok(())
}

