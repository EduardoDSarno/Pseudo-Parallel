use std::collections::VecDeque;

use crate::market_data::{
    constans::FIRST_CANDLE_INDEX,
    engine::Engine,
    hyperliquid::protocols::rest::RestResponse,
    types::{Candle, CandleKey},
};

impl Engine {
    /* This function has the job of seeding our The candle data with the historical previous max_closed_candles
    candles, so it becomes a hot start insated of a cold one*/
    pub fn seed_candles(&mut self, mut candles: VecDeque<Candle>) -> Result<(), String> {
        // Data not passed
        if candles.is_empty() {
            let err = "cannot seed engine with empty candle buffer".to_string();
            tracing::error!(error = %err, "Seed candles failed");
            return Err(err);
        }

        // We use this so we can get the exact number of candles we need for warm up
        if candles.len() < self.max_closed_candles {
            let err = format!(
                "cannot seed engine with {} candles, expected at least {}",
                candles.len(),
                self.max_closed_candles
            );
            tracing::error!(received = candles.len(), expected = self.max_closed_candles, error = %err, "Seed candles failed");
            return Err(err);
        }

        let candle_key = CandleKey::new(
            candles[FIRST_CANDLE_INDEX].coin.clone(),
            candles[FIRST_CANDLE_INDEX].interval.clone(),
        );

        // Using a guard to make sure we just have the exact amount of candles we want
        while candles.len() > self.max_closed_candles {
            candles.pop_front();
        }

        let last_open_time = candles.back().unwrap().open_time_ms;
        let live = candles.back().unwrap().clone();

        self.buffers.insert(candle_key.clone(), candles);
        self.set_last_seen(candle_key.clone(), live);

        tracing::info!(
            coin = ?candle_key.coin,
            interval = ?candle_key.interval,
            len = self.buffers.get(&candle_key).map(|b| b.len()).unwrap_or(0),
            last_seen_open_time = last_open_time,
            "Candle buffer seeded"
        );
        Ok(())
    }

    /* This is a wrapper for seed candles to handle a vector of responsed insated of one only */
    pub fn seed_from_rest_responses(&mut self, responses: Vec<RestResponse>) -> Result<(), String> {
        tracing::info!(
            responses = responses.len(),
            "Seeding engine from REST responses"
        );

        for response in responses {
            match response {
                RestResponse::CandleSnapshot(candles) => {
                    self.seed_candles(VecDeque::from(candles))?;
                }
            }
        }

        Ok(())
    }

    pub fn verify_seeded_keys(&self, keys: &[CandleKey]) -> Result<(), String> {
        for key in keys {
            let closed_len = self.closed_buffer(key).map(|buf| buf.len()).unwrap_or(0);
            let has_last_seen = self.last_seen(key).is_some();

            if closed_len != self.max_closed_candles || !has_last_seen {
                let err = format!(
                    "seed verification failed for {:?}: closed_len={} expected={}, has_last_seen={}",
                    key, closed_len, self.max_closed_candles, has_last_seen
                );
                tracing::error!(error = %err, "Seed verification failed");
                return Err(err);
            }

            let last_seen_open_time = self.last_seen(key).unwrap().open_time_ms;
            tracing::info!(
                coin = ?key.coin,
                interval = ?key.interval,
                closed_len,
                max_closed_candles = self.max_closed_candles,
                last_seen_open_time,
                "Seed verified for stream"
            );
        }

        Ok(())
    }
}
