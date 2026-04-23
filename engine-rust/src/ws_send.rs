use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum Method{

    SUBSCRIBE,
    UNSUBSCRIBE
}
/*The subscription types will only contain the types we will be using, more can be added
  later on */
#[derive(Serialize, Deserialize, Debug)]
enum SubscriptionType{
    TRADES,
    CANDLE,
    L2BOOK
}

#[derive(Serialize, Deserialize, Debug)]
struct Subscription{
    sub_type: SubscriptionType,
    coin: String
}

#[derive(Serialize, Deserialize, Debug)]
struct WsSubscribe{

    method: Method,
    subscription: Subscription
}

impl Subscription{
    fn new(sub_type: SubscriptionType, coin:String) -> Result<Subscription, String>{

        if coin.trim().is_empty() 
        {
            return Err("coin cannot be empty".to_string());
        }

        let subscription = Subscription{
            sub_type: sub_type,
            coin: coin
        };
        Ok(subscription)
    }
}

impl WsSubscribe {
    fn new(sub : Subscription, method:Method)->Result<WsSubscribe, Box<dyn std::error::Error>>{

        let ws_sub = WsSubscribe{
            subscription: sub,
            method: method
        };

        Ok(ws_sub)
    }
}

enum wsMessage{
    
}