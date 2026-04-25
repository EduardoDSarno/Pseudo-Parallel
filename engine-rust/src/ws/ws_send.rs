use serde::{Deserialize, Serialize};

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
pub enum SubscriptionType{
    #[serde(rename = "trades")]
    TRADES,
    #[serde(rename = "candle")]
    CANDLE,
    #[serde(rename = "l2Book")]
    L2BOOK,
    UNKNOWK
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Intervals {
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription{
    #[serde(rename = "type")]
    sub_type: SubscriptionType,
    coin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<Intervals>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WsMessage {
    method: Method,
    subscription: Subscription
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

impl WsMessage {
    pub fn new(method: Method, subscription: Subscription) -> WsMessage {
        WsMessage {
            method,
            subscription,
        }
    }
}
