use crate::market_data::{candle::COINS, hyperliquid::{client::run_hyperliquid_client, protocols::{data_models::candle::Interval, subscribe::{
    Method,
    SubscribeToChannelReq, SubscriptionData,
}}}};
mod market_data;



#[tokio::main]
async fn main()->Result<(), Box<dyn std::error::Error>> 
{
    
    let sub_data = SubscriptionData::Candle 
    {
        coin: COINS::HYPE,
        interval: Interval::FiveMinutes,
    };

    let sub = SubscribeToChannelReq::new(Method::SUBSCRIBE, sub_data);

    let sub_message = serde_json::to_string(&sub)?;
    run_hyperliquid_client(sub_message).await?;

    Ok(())
}

