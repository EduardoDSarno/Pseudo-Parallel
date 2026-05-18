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

        // overwrites or creates our data for candles for the respective candle_key
        tracing::info!(coin = ?candle_key.coin, interval = ?candle_key.interval, len = candles.len(), "Candle buffer seeded");
        self.buffers.insert(candle_key, candles);
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
}
