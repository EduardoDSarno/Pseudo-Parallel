use serde::{Deserialize, Serialize};

use crate::market_data::{constans::COINS, hyperliquid::protocols::candle::Interval};


#[derive(Deserialize,Serialize)]
#[serde(rename_all = "lowercase")]
pub struct SubscribeToChannelReq
{
    pub(crate) sub_method: Method,
    subscription_data: SubscriptionData
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SubscriptionData {

    Candle {
        coin: COINS,
        interval: Interval,
    },
    L2Book {
        coin: COINS,
    },
    Trades {
        coin: COINS,
    },
    UserEvents {
        user: String,
    },
}
#[derive(Deserialize,Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Method{
    SUBSCRIBE,
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

