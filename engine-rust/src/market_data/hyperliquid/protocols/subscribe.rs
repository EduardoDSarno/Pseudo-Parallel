use serde::{Deserialize, Serialize};

use crate::market_data::hyperliquid::protocols::candle::Interval;


#[derive(Deserialize,Serialize)]
pub struct SubscribeToChannelReq{

    #[serde(rename = "method")]
    pub(crate) sub_method: Method,
    #[serde(rename = "subscription")]
    subscription_data: SubscriptionData
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SubscriptionData {
    #[serde(rename = "candle")]
    Candle {
        coin: String,
        interval: Interval,
    },
    #[serde(rename = "l2Book")]
    L2Book {
        coin: String,
    },
    #[serde(rename = "trades")]
    Trades {
        coin: String,
    },
    #[serde(rename = "userEvents")]
    UserEvents {
        user: String,
    },
}
#[derive(Deserialize,Serialize)]
pub enum Method{
    #[serde(rename = "subscribe")]
    SUBSCRIBE,
    #[serde(rename = "unsubscribe")]
    UNSUBSCRIBE
}
impl SubscribeToChannelReq{
    pub fn new(method: Method, sub_data :SubscriptionData)->SubscribeToChannelReq
    {
       let req = SubscribeToChannelReq{
        sub_method: method,
        subscription_data: sub_data
       };
       req
    }
}

