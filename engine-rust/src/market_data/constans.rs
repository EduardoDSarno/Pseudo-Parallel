// use serde::{Serialize, Deserialize};
pub const HYPERLIQUID_WS_URL: &str = "wss://api.hyperliquid.xyz/ws";

pub const MAX_LENGTH_CANDLE_BUFFER: usize    = 15;
pub const LAST_CANDLE_INDEX: usize           = MAX_LENGTH_CANDLE_BUFFER - 1;
pub const SECOND_TO_LAST_CANDLE_INDEX :usize = LAST_CANDLE_INDEX - 1;

pub const ATR_BREAKOUT_RATIO:f64 = 2.5;