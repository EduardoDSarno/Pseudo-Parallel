use serde::{Deserialize, Serialize};

use crate::market_data::hyperliquid::protocols::candle::Intervals;


#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription{
    #[serde(rename = "type")]
    sub_type: SubscriptionType,
    coin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<Intervals>,
}

impl Subscription{
    pub fn new(sub_type: SubscriptionType, coin:String, interval: Option<Intervals>) -> Result<Subscription, String>{

        if coin.trim().is_empty() 
        {
            return Err("coin cannot be empty".to_string());
        }

        let subscription = Subscription{
            sub_type: sub_type,
            coin: coin,
            interval: interval,
        };
        Ok(subscription)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponseData 
{
    pub method: String,
    pub subscription: SubscriptionResponseSubscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponseSubscription 
{
    #[serde(rename = "type")]
    pub sub_type: String,
    pub coin: String,
    pub interval: Option<String>, // just for candles
}


// Enums

#[derive(Serialize, Deserialize, Debug)]
pub enum Method{
    #[serde(rename = "subscribe")]
    SUBSCRIBE,
    #[serde(rename = "unsubscribe")]
    UNSUBSCRIBE
}
/*The subscription types will only contain the types we will be using, more can be added
  later on */
#[derive(Serialize, Deserialize, Debug)]
pub enum SubscriptionType
{
    #[serde(rename = "candle")]
    CANDLE,
    UNKNOWK
}
