use std::collections::HashMap;
use std::time::Instant;

use crate::market_data::{
    constans::STREAM_STALE_MULTIPLIER,
    types::{Candle, CandleKey},
};

/*The WebSocket can stay connected while one candle stream (e.g. HYPE 15m) stops sending updates.
Reconnect logic only helps when the whole socket dies. This module watches each subscribed CandleKey
and logs a warning if it goes too long without a candle. It does not fix the stream (no resubscribe, no reconnect).
It only detects and logs.*/

struct StreamEntry {
    last_candle: Option<Instant>,
    stale_warned: bool,
}

/* Tracks last candle per key and warns if a stream goes quiet too long */
pub struct CandleStreamHealth {
    connected_at: Instant,
    entries: HashMap<CandleKey, StreamEntry>,
}

impl CandleStreamHealth {
    pub fn new(candle_keys: &[CandleKey], connected_at: Instant) -> Self {
        let entries = candle_keys
            .iter()
            .map(|key| {
                (
                    key.clone(),
                    StreamEntry {
                        last_candle: None,
                        stale_warned: false,
                    },
                )
            })
            .collect();

        Self {
            connected_at,
            entries,
        }
    }

    pub fn record_candle(&mut self, candle: &Candle) {
        let key = CandleKey::create_key_from_candle(candle);
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_candle = Some(Instant::now());
            entry.stale_warned = false;
        }
    }

    pub fn check_stale(&mut self) {
        let now = Instant::now();
        for (key, entry) in &mut self.entries {
            // grace after reconnect: use connect time until first candle on this key
            let reference = entry.last_candle.unwrap_or(self.connected_at);
            let elapsed_ms = now.duration_since(reference).as_millis() as u64;
            let interval_ms = key.interval.to_ms();
            let threshold_ms = interval_ms.saturating_mul(STREAM_STALE_MULTIPLIER);

            if is_stream_stale(elapsed_ms, interval_ms, STREAM_STALE_MULTIPLIER)
                && !entry.stale_warned
            {
                tracing::warn!(
                    coin = ?key.coin,
                    interval = ?key.interval,
                    elapsed_ms,
                    threshold_ms,
                    "Candle stream stale"
                );
                entry.stale_warned = true;
            }
        }
    }
}

pub fn is_stream_stale(elapsed_ms: u64, interval_ms: u64, multiplier: u64) -> bool {
    elapsed_ms > interval_ms.saturating_mul(multiplier)
}
