use std::collections::VecDeque;

use crate::market_data::{
    engine::Engine,
    types::{Candle, CandleKey},
};

/* This structure is responsible to be a view only option to engine it will
    supply the exact market data needed for evaluation. so we don't have to
    touch or pass the Engine*/
pub struct MarketView<'a>
{
    pub key: &'a CandleKey,
    pub closed_candles: &'a VecDeque<Candle>,
    pub live_candle: &'a Candle,
}

impl Engine
{
    pub fn market_view<'a>(&'a self, key: &'a CandleKey) -> Option<MarketView<'a>>
    {
        Some(MarketView
        {
            key,
            closed_candles: self.closed_buffer(key)?,
            live_candle: self.last_seen(key)?,
        })
    }
}
