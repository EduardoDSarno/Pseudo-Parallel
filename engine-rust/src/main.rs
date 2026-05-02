use crate::market_data::{engine::Engine, hyperliquid::{hl_client::run_hyperliquid_client, protocols::subscribe::{
    Method,
    SubscribeToChannelReq, SubscriptionData,
}}, types::candle::{COINS, CandleKey, Interval}};
mod market_data;



#[tokio::main]
async fn main()->Result<(), Box<dyn std::error::Error>> 
{

    let mut engine = Engine::new();
    let candle_key = CandleKey::new(COINS::HYPE, Interval::M5);
    let sub_data = SubscriptionData::Candle{candle_key};

    let sub = SubscribeToChannelReq::new(Method::SUBSCRIBE, sub_data);

    let sub_message = serde_json::to_string(&sub)?;
    run_hyperliquid_client(sub_message, &mut engine).await?;

    Ok(())
}

