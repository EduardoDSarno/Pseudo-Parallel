use crate::market_data::{
    coordinator::MarketDataCoordinator,
    signal::event::Event,
    types::{Candle, CandleKey},
};

impl MarketDataCoordinator
{
    pub fn handle_candle(&mut self, candle: Candle)
    {
        let live_candle = candle.clone();
        let candle_key = CandleKey::new(candle.coin.clone(), candle.interval.clone());

        if let Some(last) = self.engine.last_seen(&candle_key)
        {
            if last.open_time_ms != candle.open_time_ms
            {
                let closed = last.clone();
                tracing::debug!(coin = ?closed.coin, interval = ?closed.interval, open_time = closed.open_time_ms, "Candle closed and added to buffer");
                self.engine.push_closed_candle(candle_key.clone(), closed);
            }
        }

        self.engine.set_last_seen(candle_key, candle);

        if let Some(alert) = self.evaluator.evaluate_live_breakout(&self.engine, &live_candle)
        {
            let Event::ATR { atr, live_tr, ratio, spike_level, open_time_ms } = alert.event;

            tracing::info!
            (
                coin = ?alert.key.coin,
                interval = ?alert.key.interval,
                open_time = open_time_ms,
                atr = atr,
                live_tr = live_tr,
                ratio = ratio,
                spike_level = spike_level,
                "LIVE BREAKOUT detected"
            );
        }
    }
}
