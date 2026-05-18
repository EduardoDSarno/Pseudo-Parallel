
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use crate::market_data::{constans::HYPERLIQUID_WS_URL, coordinator::MarketDataCoordinator, hyperliquid::protocols::{inbound::InboundMessage, subscribe::SubscribeToChannelReq}, types::Candle};



pub async fn run_hyperliquid_client(subs: Vec<SubscribeToChannelReq>, coordinator: &mut MarketDataCoordinator) -> Result<(), Box<dyn std::error::Error>>
{
    tracing::info!(subscriptions = subs.len(), "Starting Hyperliquid client");

    // connect with hype Ws
    let mut ws_stream = connect_ws_hl().await?;

    // Loop through messages, serialize and send it
    for sub in &subs 
    {
        let msg = serde_json::to_string(sub)?;
        tracing::debug!(subscription = ?sub, "Sending subscription request");
        ws_stream.send(Message::Text(msg)).await?;
    }

    tracing::info!(subscriptions = subs.len(), "All subscription requests sent");


    while let Some(result) =  ws_stream.next().await
    {
        // It will return false when closed
        if !read_message(result, coordinator)
        {
            break;
        }
    }

    tracing::warn!("Hyperliquid client stopped reading messages");
    Ok(())
}

/* This function it returns a websocketsream conneciton with hyperlquid */
async fn connect_ws_hl() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>
{
    tracing::info!(url = HYPERLIQUID_WS_URL, "Connecting to Hyperliquid WS");

    let (ws_stream, response) = connect_async(HYPERLIQUID_WS_URL)
        .await
        .inspect_err(|err| tracing::error!(url = HYPERLIQUID_WS_URL, error = %err, "Connection failed"))?;

    tracing::info!(status = %response.status(), "Connected to Hyperliquid WS");

    Ok(ws_stream)
}

/* THis function will read the message and try to match (if successfully received text)
    with one of our message inbounds otherwise it will match with different types of responses
     */
fn read_message(result: Result<Message, tokio_tungstenite::tungstenite::Error>, coordinator: &mut MarketDataCoordinator) -> bool
{
    match result
    {
        Ok(Message::Text(text)) => 
        {
            let deserialized = serde_json::from_str::<InboundMessage>(&text);
            let _ = match_response(deserialized, coordinator);
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
        Ok(message) =>
        {
            tracing::warn!(message = ?message, "Unexpected WS message type");
            true
        },
        Err(err) =>
        {
            tracing::error!(error = %err, "WebSocket message error");
            true
        },

    }
    
}

/* THis function is soly responsible for matching the message with one of our inbounds streams */
fn match_response(message_response: Result<InboundMessage, serde_json::Error>, coordinator: &mut MarketDataCoordinator) -> Result<(), Box<dyn std::error::Error>>
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
            let candle = Candle::try_from(candle_hl)
                .inspect_err(|err| tracing::error!(error = %err, "Could not convert inbound candle"))?;
            coordinator.handle_candle(candle);
            Ok(())
        }
        Ok(InboundMessage::Error(msg)) => 
        {
            tracing::error!(msg = %msg, "Server error");
            Err(msg.into())
        }
        Err(e) =>
        {
            tracing::error!(error = %e, "Could not parse inbound message");
            Err(e.into())
        }
    }
}
