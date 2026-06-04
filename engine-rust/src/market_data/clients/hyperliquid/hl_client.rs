use std::time::Duration;

use crate::market_data::{
    clients::hyperliquid::{
        protocols::{inbound::InboundMessage, subscribe::subscribe_candle},
        stream_health::CandleStreamHealth,
    },
    constans::{HYPERLIQUID_WS_URL, WS_MAX_CONSECUTIVE_MESSAGE_ERRORS},
    coordinator::MarketUpdate,
    types::{Candle, CandleKey},
};
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/* This file has the simple job of creatinga hyperliquid connection.
    deconthing this message, and match it with match_inbound that will return
    possibly a market update */
#[derive(Debug, PartialEq, Eq)]
pub enum WsReadAction {
    Continue,
    MessageOk,
    MessageError,
    Reconnect,
}

/* Connect to Hyperliquid WS. */
pub async fn connect_ws_hl(
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> 
{
 
    tracing::info!(url = HYPERLIQUID_WS_URL, "Connecting to Hyperliquid WS");

    let (ws_stream, response) = 
    connect_async(HYPERLIQUID_WS_URL).await.inspect_err
    (
        |err| tracing::error!(url = HYPERLIQUID_WS_URL, error = %err, "Connection failed"),
    )?;

    tracing::info!(status = %response.status(), "Connected to Hyperliquid WS");

    Ok(ws_stream)
}

/* Send candle stream subscriptions for each key. made from coin and interval */
pub async fn send_subscriptions(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    candle_keys: &[CandleKey],
) -> Result<(), Box<dyn std::error::Error>> 
{

    for key in candle_keys 
    {
        // create subscription message struct
        let sub = subscribe_candle(key.coin, key.interval.clone());

        let msg = serde_json::to_string(&sub)?;
        tracing::debug!(subscription = ?sub, "Sending subscription request");
        ws_stream.send(Message::Text(msg)).await?;
    }
    Ok(())
}

/* Exponential backoff with ceiling before reconnect retry. */
pub fn next_backoff(current: Duration) -> Duration {
    use crate::market_data::constans::{WS_RECONNECT_BACKOFF_MULTIPLIER, WS_RECONNECT_MAX_MS};

    let doubled = current.saturating_mul(WS_RECONNECT_BACKOFF_MULTIPLIER);
    let max = Duration::from_millis(WS_RECONNECT_MAX_MS);
    if doubled > max {
        max
    } else {
        doubled
    }
}

/* Decode one WS frame into a read action and optional MarketUpdate for the coordinator. */
pub fn decode_ws_message(
    result: Result<Message, tokio_tungstenite::tungstenite::Error>,
    health: &mut CandleStreamHealth,
) -> (WsReadAction, Option<MarketUpdate>) 
{
    match result {
        Ok(Message::Text(text)) => {
            let deserialized = serde_json::from_str::<InboundMessage>(&text);
            match match_inbound(deserialized, health) {
                Ok(update) => (WsReadAction::MessageOk, update),
                Err(_) => (WsReadAction::MessageError, None),
            }
        }
        Ok(Message::Ping(_)) => {
            tracing::trace!("Received ping");
            (WsReadAction::Continue, None)
        }
        Ok(Message::Pong(_)) => {
            tracing::trace!("Received pong");
            (WsReadAction::Continue, None)
        }
        Ok(Message::Close(close_frame)) => {
            tracing::warn!(frame = ?close_frame, "WebSocket closed");
            (WsReadAction::Reconnect, None)
        }
        Ok(message) => {
            tracing::warn!(message = ?message, "Unexpected WS message type");
            (WsReadAction::Continue, None)
        }
        Err(err) => {
            tracing::error!(error = %err, "WebSocket message error");
            (WsReadAction::Reconnect, None)
        }
    }
}

pub fn apply_message_error_policy(
    action: WsReadAction,
    consecutive_message_errors: &mut u32,
) -> WsReadAction {
    match action {
        WsReadAction::MessageOk => {
            *consecutive_message_errors = 0;
            WsReadAction::Continue
        }
        WsReadAction::MessageError => {
            *consecutive_message_errors += 1;

            if *consecutive_message_errors >= WS_MAX_CONSECUTIVE_MESSAGE_ERRORS {
                tracing::error!(
                    consecutive_message_errors,
                    max = WS_MAX_CONSECUTIVE_MESSAGE_ERRORS,
                    "Too many WebSocket message errors, reconnecting"
                );
                return WsReadAction::Reconnect;
            }

            tracing::warn!(
                consecutive_message_errors,
                max = WS_MAX_CONSECUTIVE_MESSAGE_ERRORS,
                "WebSocket message error counted"
            );
            WsReadAction::Continue
        }
        WsReadAction::Continue => WsReadAction::Continue,
        WsReadAction::Reconnect => WsReadAction::Reconnect,
    }
}

/* Match inbound JSON to optional MarketUpdate — no runtime access here. */
fn match_inbound(
    message_response: Result<InboundMessage, serde_json::Error>,
    health: &mut CandleStreamHealth,
) -> Result<Option<MarketUpdate>, Box<dyn std::error::Error>> {
    match message_response 
    {
        // if the message match a subscription response, everything is fine
        Ok(InboundMessage::SubscriptionResponse(response)) => {
            tracing::info!(
                method = ?response.method,
                subscription = ?response.subscription,
                "Subscription confirmed"
            );
            Ok(None)
        }
        Ok(InboundMessage::Candle(candle_hl)) => {
            let candle = Candle::try_from(candle_hl).inspect_err(
                |err| tracing::error!(error = %err, "Could not convert inbound candle"),
            )?;
            health.record_candle(&candle);
            Ok(Some(MarketUpdate::Candle(candle)))
        }
        Ok(InboundMessage::Error(msg)) => {
            tracing::error!(msg = %msg, "Server error");
            Err(msg.into())
        }
        Err(e) => {
            tracing::error!(error = %e, "Could not parse inbound message");
            Err(e.into())
        }
    }
}
