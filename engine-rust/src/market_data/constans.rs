// use serde::{Serialize, Deserialize};
pub const HYPERLIQUID_WS_URL:   &str = "wss://api.hyperliquid.xyz/ws";
pub const HYPERLIQUID_REST_URL: &str = "https://api.hyperliquid.xyz/info";

pub const MAX_LENGTH_CANDLE_BUFFER: usize    = 15;
pub const LAST_CANDLE_INDEX: usize           = MAX_LENGTH_CANDLE_BUFFER - 1;
pub const SECOND_TO_LAST_CANDLE_INDEX :usize = LAST_CANDLE_INDEX - 1;

pub const ONE_MINUTE_MS: u64 = 60 * 1000;
pub const M1_INTERVAL_MS: u64 = ONE_MINUTE_MS;
pub const M5_INTERVAL_MS: u64 = 5 * ONE_MINUTE_MS;
pub const M15_INTERVAL_MS: u64 = 15 * ONE_MINUTE_MS;
pub const H1_INTERVAL_MS: u64 = 60 * ONE_MINUTE_MS;

pub const ATR_BREAKOUT_RATIO:f64 = 2.5;