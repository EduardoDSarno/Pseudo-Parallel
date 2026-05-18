use crate::market_data::{
    constans::MIN_VALID_ATR,
    engine::MarketView,
    signal::indicators::atr::{calculate_average_true_range, calculate_true_range},
    types::CandleKey,
};

pub struct AtrEvaluation
{
    pub key: CandleKey,
    pub atr: f64,
    pub live_tr: f64,
    pub ratio: f64,
    pub open_time_ms: u64,
}

pub struct AtrEvaluator
{
    max_closed_candles: usize,
}

impl AtrEvaluator
{
    pub fn new(max_closed_candles: usize) -> Self
    {
        AtrEvaluator
        {
            max_closed_candles,
        }
    }

    pub fn evaluate_live_atr(&self, view: MarketView<'_>) -> Option<AtrEvaluation>
    {
        if view.closed_candles.len() < self.max_closed_candles
        {
            tracing::debug!(coin = ?view.key.coin, interval = ?view.key.interval, len = view.closed_candles.len(), max = self.max_closed_candles, "Buffer warming up");
            return None;
        }

        let atr_input = view.closed_candles.iter().cloned().collect();
        let atr = calculate_average_true_range(&atr_input)?;
        if atr <= MIN_VALID_ATR
        {
            tracing::warn!(coin = ?view.key.coin, interval = ?view.key.interval, atr = atr, "ATR is not valid for live evaluation");
            return None;
        }

        let latest_closed = view.closed_candles.back()?;
        let live_tr = calculate_true_range(latest_closed, view.live_candle);

        Some(AtrEvaluation
        {
            key: view.key.clone(),
            atr,
            live_tr,
            ratio: live_tr / atr,
            open_time_ms: view.live_candle.open_time_ms,
        })
    }
}
