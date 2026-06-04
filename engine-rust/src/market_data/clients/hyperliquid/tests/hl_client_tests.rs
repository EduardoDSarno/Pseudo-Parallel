use std::time::Instant;

use crate::market_data::{
    clients::hyperliquid::{
        hl_client::{apply_message_error_policy, decode_ws_message, WsReadAction},
        stream_health::CandleStreamHealth,
    },
    types::{CandleKey, Coins, Interval},
};
use tokio_tungstenite::tungstenite::Message;

#[test]
fn message_error_counter_reconnects_at_max() {
    let mut count = crate::market_data::constans::WS_MAX_CONSECUTIVE_MESSAGE_ERRORS - 1;

    let action = apply_message_error_policy(WsReadAction::MessageError, &mut count);

    assert_eq!(action, WsReadAction::Reconnect);
}

#[test]
fn message_ok_resets_error_counter() {
    let mut count = 3;

    let action = apply_message_error_policy(WsReadAction::MessageOk, &mut count);

    assert_eq!(action, WsReadAction::Continue);
    assert_eq!(count, 0);
}

#[test]
fn parse_error_returns_message_error() {
    let keys = [CandleKey::new(Coins::HYPE, Interval::M5)];
    let mut health = CandleStreamHealth::new(&keys, Instant::now());

    let (action, update) = decode_ws_message(
        Ok(Message::Text("{not valid json".to_string())),
        &mut health,
    );

    assert_eq!(action, WsReadAction::MessageError);
    assert!(update.is_none());
}
