// use serde::{Serialize, Deserialize};
pub const HYPERLIQUID_WS_URL: &str = "wss://api.hyperliquid.xyz/ws";
pub const HYPERLIQUID_REST_URL: &str = "https://api.hyperliquid.xyz/info";

pub const DEFAULT_MAX_CLOSED_CANDLES: usize = 20;

pub const ONE_MINUTE_MS: u64 = 60 * 1000;
pub const M1_INTERVAL_MS: u64 = ONE_MINUTE_MS;
pub const M5_INTERVAL_MS: u64 = 5 * ONE_MINUTE_MS;
pub const M15_INTERVAL_MS: u64 = 15 * ONE_MINUTE_MS;
pub const H1_INTERVAL_MS: u64 = 60 * ONE_MINUTE_MS;

pub const ATR_BREAKOUT_RATIO: f64 = 2.5;
pub const LIVE_ATR_DEBUG_RATIO: f64 = 0.8;
pub const _MIN_VALID_ATR: f64 = 0.0;

pub const NO_SPIKE_LEVEL: u64 = 0;
pub const FIRST_CANDLE_INDEX: usize = 0;
/* Minimum closed candles needed to compute one true-range pair for ATR */
pub const MIN_CANDLES_FOR_ATR: usize = 2;

pub const PRICE_SCALE: f64 = 100_000_000.0;
