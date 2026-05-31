use crate::market_data::{
    engine::Engine,
    types::{Candle, CandleKey},
};

pub struct IngestedCandleSnapshot {
    pub candle_key: CandleKey,
    pub close_price: f64,
}

/* This function has the job of updating the engine, it builds a candle key 
If the previous candle for that key closed → push it into the closed-candle buffer.
 Store this candle as last seen for that key. */
pub fn apply_candle(engine: &mut Engine, candle: Candle) -> IngestedCandleSnapshot {
    let candle_key = CandleKey::create_key_from_candle(&candle);

    if let Some(last) = engine.last_seen(&candle_key) {
        if last.open_time_ms != candle.open_time_ms {
            let closed = last.clone();
            tracing::debug!(
                coin = ?closed.coin,
                interval = ?closed.interval,
                open_time = closed.open_time_ms,
                "Candle closed and added to buffer"
            );
            engine.push_closed_candle(candle_key.clone(), closed);
        }
    }

    engine.set_last_seen(candle_key.clone(), candle.clone());

    IngestedCandleSnapshot {
        candle_key,
        close_price: candle.close_price,
    }
}
