use std::collections::VecDeque;

use crate::market_data::{
    candle_store::CandleStore,
    clients::hyperliquid::protocols::rest::RestResponse,
    constans::FIRST_CANDLE_INDEX,
    types::{Candle, CandleKey},
};

impl CandleStore {
    /* This function seeds assuming all received candles are closed already. Tests and non-REST callers can use it. */
    #[cfg(test)]
    pub fn seed_candles(&mut self, candles: VecDeque<Candle>) -> Result<(), String> {
        self.seed_candles_at(candles, u64::MAX)
    }

    /* REST can return the candle forming right now. This keeps only closed candles in the buffer
    and stores the forming candle in last_seen so the candle store has one source of truth. */
    pub fn seed_candles_at(
        &mut self,
        mut candles: VecDeque<Candle>,
        seed_end_time: u64,
    ) -> Result<(), String> {
        // Data not passed
        if candles.is_empty() {
            let err = "cannot seed candle store with empty candle buffer".to_string();
            tracing::error!(error = %err, "Seed candles failed");
            return Err(err);
        }

        let candle_key = CandleKey::new(
            candles[FIRST_CANDLE_INDEX].coin.clone(),
            candles[FIRST_CANDLE_INDEX].interval.clone(),
        );
        let received_len = candles.len();
        let mut closed_candles = VecDeque::new();
        let mut live_candle = None;

        while let Some(candle) = candles.pop_front() {
            if candle.close_time_ms < seed_end_time {
                closed_candles.push_back(candle);
            } else if candle.open_time_ms <= seed_end_time && seed_end_time <= candle.close_time_ms
            {
                live_candle = Some(candle);
            }
        }

        // We use this so we can get the exact number of closed candles we need for warm up
        if closed_candles.len() < self.max_closed_candles {
            let err = format!(
                "cannot seed candle store with {} closed candles, expected at least {}",
                closed_candles.len(),
                self.max_closed_candles
            );
            tracing::error!(
                received = received_len,
                closed = closed_candles.len(),
                expected = self.max_closed_candles,
                seed_end_time,
                error = %err,
                "Seed candles failed"
            );
            return Err(err);
        }

        // Using a guard to make sure we just have the exact amount of candles we want
        while closed_candles.len() > self.max_closed_candles {
            closed_candles.pop_front();
        }

        let live = live_candle
            .clone()
            .unwrap_or_else(|| closed_candles.back().unwrap().clone());
        let last_open_time = live.open_time_ms;
        let had_forming_candle = live_candle.is_some();

        self.buffers.insert(candle_key.clone(), closed_candles);
        self.set_last_seen(candle_key.clone(), live);

        tracing::info!(
            coin = ?candle_key.coin,
            interval = ?candle_key.interval,
            received = received_len,
            len = self.buffers.get(&candle_key).map(|b| b.len()).unwrap_or(0),
            had_forming_candle,
            last_seen_open_time = last_open_time,
            "Candle buffer seeded"
        );
        Ok(())
    }

    /* This is a wrapper for seed candles to handle a vector of responsed insated of one only */
    pub fn seed_from_rest_responses(
        &mut self,
        responses: Vec<RestResponse>,
        seed_end_time: u64,
    ) -> Result<(), String> {
        tracing::info!(
            responses = responses.len(),
            seed_end_time,
            "Seeding candle store from REST responses"
        );

        for response in responses {
            match response {
                RestResponse::CandleSnapshot(candles) => {
                    self.seed_candles_at(VecDeque::from(candles), seed_end_time)?;
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
