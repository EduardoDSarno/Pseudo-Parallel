use serde::{Deserialize, Serialize};

use crate::market_data::types::candle::Coins;
use crate::market_data::types::candle::CandleKey;
use crate::market_data::types::candle::Interval;


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

    Candle 
    {
        #[serde(flatten)]
        candle_key: CandleKey
    },
    L2Book {
        coin: Coins,
    },
    Trades {
        coin: Coins,
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
    pub fn new(method: Method, sub_data : SubscriptionData)->SubscribeToChannelReq
    {
       let req = SubscribeToChannelReq{
        method: method,
        subscription: sub_data
       };
       req
    }
}

/* This function is a wrapper to be  to subscribe for a hyperliquid candle stream*/
pub fn subscribe_candle(coin: Coins, interval: Interval) -> SubscribeToChannelReq 
{
    SubscribeToChannelReq::new
    (
        Method::SUBSCRIBE,
        SubscriptionData::Candle 
        {
            candle_key: CandleKey::new(coin, interval),
        },
    )
}

