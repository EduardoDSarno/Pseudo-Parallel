mod market_data;

use crate::market_data::hyperliquid::protocols::candle::{Intervals, WsMessageRecv};
use crate::market_data::hyperliquid::protocols::message::WsMessage;
use crate::market_data::hyperliquid::protocols::subscribe::{Method, Subscription, SubscriptionType};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main()->Result<(), Box<dyn std::error::Error>> 
{
    // response won't be used since we are just receiving and processing data, not sending
    let (mut ws, _response) = connect_async("wss://api.hyperliquid.xyz/ws").await?;

    let sub = Subscription::new(SubscriptionType::CANDLE, "HYPE".to_string(), Some(Intervals::FiveMinutes));
    let message = WsMessage::new(Method::SUBSCRIBE, sub?);

    let message_json = serde_json::to_string(&message).unwrap();
    println!("serialized = {}", message_json);

    ws.send(Message::Text(message_json)).await?;

    while let Some(recv) = ws.next().await
    {
        let recv = recv?;

        match recv 
        {
            Message::Text(s)=>
            {
                
                let deserialized: WsMessageRecv = serde_json::from_str(&s)?;
                println!("Message: {:#?}", deserialized);
                
            }
            Message::Ping(s)=>
            {
                // auto respond pings
                ws.send(Message::Pong(s)).await?;
            }
            Message::Close(_) => {
                break;  // exit the receive loop
            }
            _ =>{}
        }
    }
   

    Ok(())
}

