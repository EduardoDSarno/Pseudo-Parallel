use crate::market_data::{
    candle_store::CandleStore,
    types::{Candle, CandleKey},
};

pub struct IngestedCandleSnapshot {
    pub candle_key: CandleKey,
    pub previous_price: Option<f64>,
    pub current_price: f64,
}

/* This function has the job of updating the candle store, it builds a candle key
If the previous candle for that key closed → push it into the closed-candle buffer.
 Store this candle as last seen for that key. */
pub fn apply_candle(engine: &mut CandleStore, candle: Candle) -> IngestedCandleSnapshot {
    let candle_key = CandleKey::create_key_from_candle(&candle);
    let previous_price = engine
        .last_seen(&candle_key)
        .map(|previous| previous.close_price);

    if let Some(last) = engine.last_seen(&candle_key) {
        if last.open_time_ms != candle.open_time_ms {
            let closed = last.clone();
            let already_in_buffer = engine
                .closed_buffer(&candle_key)
                .and_then(|buf| buf.back())
                .map(|tail| tail.open_time_ms == closed.open_time_ms)
                .unwrap_or(false);

            if !already_in_buffer {
                tracing::debug!(
                    coin = ?closed.coin,
                    interval = ?closed.interval,
                    open_time = closed.open_time_ms,
                    "Candle closed and added to buffer"
                );
                engine.push_closed_candle(candle_key.clone(), closed);
            }
        }
    }

    engine.set_last_seen(candle_key.clone(), candle.clone());

    IngestedCandleSnapshot {
        candle_key,
        previous_price,
        current_price: candle.close_price,
    }
}
