use serde::{Deserialize, Serialize};

use crate::market_data::constans::COINS;
use crate::market_data::hyperliquid::protocols::data_models::candle::Interval;


#[derive(Deserialize,Serialize)]
#[serde(rename_all = "lowercase")]
pub struct SubscribeToChannelReq
{
    method: Method,
    subscription: SubscriptionData
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
#[derive(Deserialize,Serialize, Debug, Clone,)]
#[serde(rename_all = "lowercase")]
pub enum Method{
    SUBSCRIBE,
    UNSUBSCRIBE
}
impl SubscribeToChannelReq
{
    pub fn new(method: Method, sub_data :SubscriptionData)->SubscribeToChannelReq
    {
       let req = SubscribeToChannelReq{
        method: method,
        subscription: sub_data
       };
       req
    }
}

