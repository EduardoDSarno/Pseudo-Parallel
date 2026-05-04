
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use crate::market_data::{constans::HYPERLIQUID_WS_URL, engine::Engine, hyperliquid::protocols::{inbound::InboundMessage, subscribe::SubscribeToChannelReq}, signal::event::handle_candle_event, types::candle::Candle};



pub async fn run_hyperliquid_client(subs: Vec<SubscribeToChannelReq>, engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> 
{
    // connect with hype Ws
    let mut ws_stream = connect_ws_hl().await?;

    // Loop through messages, serialize and send it
    for sub in &subs 
    {
        let msg = serde_json::to_string(sub)?;
        ws_stream.send(Message::Text(msg)).await?;
    }


    while let Some(result) =  ws_stream.next().await
    {
        // It will return false when closed
        if !read_message(result, engine)
        {
            break;
        }
    }

    Ok(())
}

/* This function it returns a websocketsream conneciton with hyperlquid */
async fn connect_ws_hl() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>
{
    let (ws_stream, response) = 
    connect_async(HYPERLIQUID_WS_URL).
    await.
    expect("Connection Failed");

    tracing::info!(status = %response.status(), "Connected to Hyperliquid WS");

    Ok(ws_stream)
}

/* THis function will read the message and try to match (if successfully received text)
    with one of our message inbounds otherwise it will match with different types of responses
     */
fn read_message(result: Result<Message, tokio_tungstenite::tungstenite::Error>, engine: &mut Engine,) -> bool
{
    match result
    {
        Ok(Message::Text(text)) => 
        {
            let deserialized = serde_json::from_str::<InboundMessage>(&text);
            let _= match_response(deserialized, engine);
            true
        }
        // Tokio tungstain handles automatically
        Ok(Message::Ping(_message)) =>
        {
            tracing::trace!("Received ping");
            true
        }
        Ok(Message::Pong(_message))=>
        {
            tracing::trace!("Received pong");
            true
        }
        Ok(Message::Close(close_frame)) => 
        {
            tracing::warn!(frame = ?close_frame, "WebSocket closed");
            false
        }
        _ => 
        {
            tracing::warn!("Unexpected WS message type");
            true
        },

    }
    
}

/* THis function is soly responsible for matching the message with one of our inbounds streams */
fn match_response(message_response: Result<InboundMessage, serde_json::Error>, engine: &mut Engine,) -> Result<(), Box<dyn std::error::Error>>
{

    match message_response
    {
        Ok(InboundMessage::SubscriptionResponse(response))=>
        {
            tracing::info!(method = ?response.method, subscription = ?response.subscription, "Subscription confirmed");
            Ok(())
        }

        Ok(InboundMessage::Candle(candle_hl)) => 
        {
            let candle = Candle::try_from(candle_hl)?;
            handle_candle_event(engine, candle);
            Ok(())
        }
        Ok(InboundMessage::Error(msg)) => 
        {
            tracing::error!(msg = %msg, "Server error");
            Err(msg.into())
        }
        Err(e) =>  Err(e.into())
    }
}
