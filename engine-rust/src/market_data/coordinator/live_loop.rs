use std::{error::Error, time::Duration};

use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::market_data::{
    alert_subscriptions::command::SubscriptionManager,
    clients::hyperliquid::{
        hl_client::{
            apply_message_error_policy, connect_ws_hl, decode_ws_message, next_backoff,
            send_subscriptions, WsReadAction,
        },
        stream_health::CandleStreamHealth,
    },
    constans::{STREAM_HEALTH_CHECK_INTERVAL_MS, WS_RECONNECT_INITIAL_MS},
    coordinator::MarketUpdate,
    runtime::MarketDataRuntime,
    types::CandleKey,
};

struct HyperliquidSession {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    health: CandleStreamHealth,
    health_tick: tokio::time::Interval,
}

/* Live market data loop — WS candles, stream health, and signal subscription updates. */
pub async fn run_live(
    runtime: &mut MarketDataRuntime,
    candle_keys: &[CandleKey],
    mut subscription_receiver: mpsc::Receiver<SubscriptionManager>, // passing receiver end of stream
) -> Result<(), Box<dyn Error>> {
    tracing::info!(
        streams = candle_keys.len(),
        "Starting live coordinator loop"
    );

    // exponation backoff for connection failling
    let mut backoff = Duration::from_millis(WS_RECONNECT_INITIAL_MS);

    /*This outer loop handles session and reconnection with open_hyperliquid_session
    taking care of connect + subscribe + health per session if fails it retries with inner backoff */
    loop {
        let Some(session) = open_hyperliquid_session(candle_keys, &mut backoff).await else {
            continue;
        };

        // creating new session
        let HyperliquidSession {
            mut ws_stream,
            mut health,
            mut health_tick,
        } = session;

        let mut consecutive_message_errors = 0; // counter for errors
        loop {
            /* This is reached just when connection is successefull */
            tokio::select! {
                // this stream reads next message decodes into an action (cotinue, reconnect etc)
                // and check for a market update, if so it process this update
                result = ws_stream.next() => {
                    match result {
                        None => {
                            tracing::info!("WebSocket stream ended");
                            break;
                        }
                        Some(msg_result) => {
                            // decode into some marketupdate
                            let (action, update) = decode_ws_message(msg_result, &mut health);

                            // if any found process it
                            if let Some(market_update) = update
                            {
                                runtime.process(market_update);
                            }
                            // control states (reconnection, and errors)
                            // update backoff if nescessary
                            match apply_message_error_policy(action, &mut consecutive_message_errors)
                            {
                                WsReadAction::Continue => {}
                                WsReadAction::MessageOk => {}
                                WsReadAction::MessageError => {}
                                WsReadAction::Reconnect => break,
                            }
                        }
                    }
                }
                // this arm does health checks every STREAM_HEALTH_CHECK_INTERVAL_MS
                _ = health_tick.tick() => {
                    health.check_stale();
                }
                // That arm waits on the receiver side of the mpsc channel from main
                // if some menager a subscription was sent
                sub = subscription_receiver.recv() =>
                {
                    match sub {
                        Some(manager) => {
                            runtime.process(MarketUpdate::Subscription(manager));
                        }
                        None => tracing::warn!("subscription channel closed"),
                    }
                }
            }
        }

        /* if connection breaks in inner loop
        Sleep before trying a new session
        Increase backoff for the next outer iteration (so repeated dropouts back off more)
        also reset on success */
        tracing::info!("Hyperliquid session ended, reconnecting after backoff");
        tokio::time::sleep(backoff).await;
        backoff = next_backoff(backoff);
    }
}

/* Connect to HL, send candle subs, and build health tracking — None means retry outer loop. */
async fn open_hyperliquid_session(
    candle_keys: &[CandleKey],
    backoff: &mut Duration,
) -> Option<HyperliquidSession> {
    let mut ws_stream = match connect_ws_hl().await // wait for conenction
    {
        Ok(stream) => stream,
        Err(err) => {
            tracing::error!(error = %err, "Connection failed, will retry");
            // increase time for reconnection
            tokio::time::sleep(*backoff).await;
            *backoff = next_backoff(*backoff);
            return None;
        }
    };

    // trying sending subscriptions, if fail. Incrase backoff
    if let Err(err) = send_subscriptions(&mut ws_stream, candle_keys).await {
        tracing::error!(error = %err, "Subscribe failed, will retry");
        tokio::time::sleep(*backoff).await;
        *backoff = next_backoff(*backoff);
        return None;
    }

    tracing::info!(
        subscriptions = candle_keys.len(),
        "All candle subscription requests sent"
    );

    // reseting when passed
    *backoff = Duration::from_millis(WS_RECONNECT_INITIAL_MS);

    let connected_at = std::time::Instant::now();
    let health = CandleStreamHealth::new(candle_keys, connected_at);
    let mut health_tick =
        tokio::time::interval(Duration::from_millis(STREAM_HEALTH_CHECK_INTERVAL_MS));
    health_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    Some(HyperliquidSession {
        ws_stream,
        health,
        health_tick,
    })
}
