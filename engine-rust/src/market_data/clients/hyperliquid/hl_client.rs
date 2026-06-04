use std::time::Duration;

use crate::market_data::{
    clients::hyperliquid::{
        protocols::{inbound::InboundMessage, subscribe::subscribe_candle},
        stream_health::CandleStreamHealth,
    },
    constans::{
        HYPERLIQUID_WS_URL, STREAM_HEALTH_CHECK_INTERVAL_MS, WS_MAX_CONSECUTIVE_MESSAGE_ERRORS,
        WS_RECONNECT_BACKOFF_MULTIPLIER, WS_RECONNECT_INITIAL_MS, WS_RECONNECT_MAX_MS,
    },
    coordinator::MarketUpdate,
    runtime::MarketDataRuntime,
    types::{Candle, CandleKey},
};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug, PartialEq, Eq)]
pub enum WsReadAction {
    Continue,
    MessageOk,
    MessageError,
    Reconnect,
}

/* Outer loop: reconnect forever, resubscribe all candle keys each session */
pub async fn run_hyperliquid_client(
    candle_keys: &[CandleKey],
    runtime: &mut MarketDataRuntime,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(streams = candle_keys.len(), "Starting Hyperliquid client");

    let mut backoff = Duration::from_millis(WS_RECONNECT_INITIAL_MS);

    loop {
        let mut ws_stream = match connect_ws_hl().await {
            Ok(stream) => stream,
            Err(err) => {
                // retry reconnecting with exponantial backoff
                tracing::error!(error = %err, "Connection failed, will retry");
                tokio::time::sleep(backoff).await;
                backoff = next_backoff(backoff);
                continue;
            }
        };

        // sending subscription with keys to the
        if let Err(err) = send_subscriptions(&mut ws_stream, candle_keys).await {
            tracing::error!(error = %err, "Subscribe failed, will retry");
            tokio::time::sleep(backoff).await;
            backoff = next_backoff(backoff);
            continue;
        }

        tracing::info!(
            subscriptions = candle_keys.len(),
            "All subscription requests sent"
        );

        // reset backoff after we are live again
        backoff = Duration::from_millis(WS_RECONNECT_INITIAL_MS);

        let connected_at = std::time::Instant::now();
        let mut health = CandleStreamHealth::new(candle_keys, connected_at);
        let mut health_tick =
            tokio::time::interval(Duration::from_millis(STREAM_HEALTH_CHECK_INTERVAL_MS));
        health_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        /* this is a per connection  loop read one websocket until it dies, if fails the outer
        loop reconnects*/
        let mut consecutive_message_errors = 0;
        loop {
            tokio::select! // runs two tasks at once and handles whichever finishes first
            {
                result = ws_stream.next() => {
                    match result {
                        None => {
                            tracing::info!("WebSocket stream ended");
                            break;
                        }
                        Some(msg_result) => {
                            match apply_message_error_policy(
                                read_message(msg_result, runtime, &mut health),
                                &mut consecutive_message_errors,
                            ) {
                                WsReadAction::Continue => {}
                                WsReadAction::MessageOk => {}
                                WsReadAction::MessageError => {}
                                // ws error or close — break inner loop and reconnect
                                WsReadAction::Reconnect => break,
                            }
                        }
                    }
                }
                _ = health_tick.tick() => {
                    health.check_stale();
                }
            }
        }

        tracing::info!("Hyperliquid session ended, reconnecting after backoff");
        // outer loop — session ended, backoff then connect again
        tokio::time::sleep(backoff).await;
        backoff = next_backoff(backoff);
    }
}

/* Receives the keys and stream and send subscriptions to it */
async fn send_subscriptions(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    candle_keys: &[CandleKey],
) -> Result<(), Box<dyn std::error::Error>> {
    for key in candle_keys {
        let sub = subscribe_candle(key.coin, key.interval.clone());
        let msg = serde_json::to_string(&sub)?;
        tracing::debug!(subscription = ?sub, "Sending subscription request");
        ws_stream.send(Message::Text(msg)).await?;
    }
    Ok(())
}

/*implements exponential backoff with a ceiling for how long the client waits before trying again */
fn next_backoff(current: Duration) -> Duration {
    let doubled = current.saturating_mul(WS_RECONNECT_BACKOFF_MULTIPLIER);
    let max = Duration::from_millis(WS_RECONNECT_MAX_MS);
    if doubled > max {
        max
    } else {
        doubled
    }
}

/* This function it returns a websocketsream conneciton with hyperlquid */
async fn connect_ws_hl(
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
    tracing::info!(url = HYPERLIQUID_WS_URL, "Connecting to Hyperliquid WS");

    let (ws_stream, response) = connect_async(HYPERLIQUID_WS_URL).await.inspect_err(
        |err| tracing::error!(url = HYPERLIQUID_WS_URL, error = %err, "Connection failed"),
    )?;

    tracing::info!(status = %response.status(), "Connected to Hyperliquid WS");

    Ok(ws_stream)
}

/* THis function will read the message and try to match (if successfully received text)
with one of our message inbounds otherwise it will match with different types of responses
 */
pub fn read_message(
    result: Result<Message, tokio_tungstenite::tungstenite::Error>,
    runtime: &mut MarketDataRuntime,
    health: &mut CandleStreamHealth,
) -> WsReadAction {
    match result {
        Ok(Message::Text(text)) => {
            let deserialized = serde_json::from_str::<InboundMessage>(&text);
            match match_response(deserialized, runtime, health) {
                Ok(()) => WsReadAction::MessageOk,
                Err(_) => WsReadAction::MessageError,
            }
        }
        // Tokio tungstain handles automatically
        Ok(Message::Ping(_message)) => {
            tracing::trace!("Received ping");
            WsReadAction::Continue
        }
        Ok(Message::Pong(_message)) => {
            tracing::trace!("Received pong");
            WsReadAction::Continue
        }
        Ok(Message::Close(close_frame)) => {
            tracing::warn!(frame = ?close_frame, "WebSocket closed");
            WsReadAction::Reconnect
        }
        Ok(message) => {
            tracing::warn!(message = ?message, "Unexpected WS message type");
            WsReadAction::Continue
        }
        Err(err) => {
            tracing::error!(error = %err, "WebSocket message error");
            WsReadAction::Reconnect
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

/* THis function is soly responsible for matching the message with one of our inbounds streams */
fn match_response(
    message_response: Result<InboundMessage, serde_json::Error>,
    runtime: &mut MarketDataRuntime,
    health: &mut CandleStreamHealth,
) -> Result<(), Box<dyn std::error::Error>> {
    match message_response {
        Ok(InboundMessage::SubscriptionResponse(response)) => {
            tracing::info!(method = ?response.method, subscription = ?response.subscription, "Subscription confirmed");
            Ok(())
        }

        Ok(InboundMessage::Candle(candle_hl)) => {
            let candle = Candle::try_from(candle_hl).inspect_err(
                |err| tracing::error!(error = %err, "Could not convert inbound candle"),
            )?;
            // record before process so health sees transport even if ingest fails
            health.record_candle(&candle);
            runtime.process(MarketUpdate::Candle(candle));
            Ok(())
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
