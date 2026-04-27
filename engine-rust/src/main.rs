use crate::market_data::hyperliquid::protocols::{candle::Interval, subscribe::{
    Method,
    SubscribeToChannelReq, SubscriptionData,
}};
mod market_data;



#[tokio::main]
async fn main()->Result<(), Box<dyn std::error::Error>> 
{
    
    let sub_data = SubscriptionData::Candle 
    {
        coin: "HYPE".to_string(),
        interval: Interval::FiveMinutes,
    };

    let sub = SubscribeToChannelReq::new(Method::SUBSCRIBE, sub_data);

    let sub_message = serde_json::to_string(&sub)?;
    println!("{:#}", sub_message);

    
    Ok(())
}

