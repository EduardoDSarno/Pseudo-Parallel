// use serde::{Serialize, Deserialize};
pub const HYPERLIQUID_WS_URL: &str = "wss://api.hyperliquid.xyz/ws";

pub const WS_RECONNECT_INITIAL_MS: u64 = 1_000;
pub const WS_RECONNECT_MAX_MS: u64 = 60_000;
pub const WS_RECONNECT_BACKOFF_MULTIPLIER: u32 = 2;
pub const WS_MAX_CONSECUTIVE_MESSAGE_ERRORS: u32 = 5;

pub const STREAM_STALE_MULTIPLIER: u64 = 2;
pub const STREAM_HEALTH_CHECK_INTERVAL_MS: u64 = 30_000;

pub const DEFAULT_MAX_CLOSED_CANDLES: usize = 20;

pub const ONE_MINUTE_MS: u64 = 60 * 1000;
pub const M5_INTERVAL_MS: u64 = 5 * ONE_MINUTE_MS;

pub const PRICE_SCALE: f64 = 100_000_000.0;

pub const BUFFER_SIZE_FOR_MPSC: usize = 32;
