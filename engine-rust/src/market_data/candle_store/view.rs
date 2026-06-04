use std::collections::VecDeque;

use crate::market_data::{
    candle_store::CandleStore,
    types::{Candle, CandleKey},
};

/* This structure is responsible to be a view only option to candle store it will
supply the exact market data needed for evaluation. so we don't have to
touch or pass the CandleStore and it bundles with candle key for easy look up
and access*/
pub struct CandleView<'a> {
    pub key: &'a CandleKey,
    pub closed_candles: &'a VecDeque<Candle>,
    /// Forming bar from `last_seen`; v1 ATR ignores it (closed-only). Kept for future intrabar rules.
    #[allow(dead_code)]
    pub live_candle: &'a Candle,
}

impl CandleStore {
    pub fn market_view<'a>(&'a self, key: &'a CandleKey) -> Option<CandleView<'a>> {
        Some(CandleView {
            key,
            closed_candles: self.closed_buffer(key)?,
            live_candle: self.last_seen(key)?,
        })
    }
}
