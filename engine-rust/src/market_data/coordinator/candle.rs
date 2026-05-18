use crate::market_data::{
    coordinator::MarketDataCoordinator,
    signal::event::Event,
    types::{Candle, CandleKey},
};

impl MarketDataCoordinator
{
    /* This function is used to handle new incoming candles, by adding them to VECDEQUE
        and checking for breakout*/
    pub fn handle_candle(&mut self, candle: Candle)
    {
        let candle_key = CandleKey::create_key_from_candle(&candle);

        if let Some(last) = self.engine.last_seen(&candle_key)
        {
            if last.open_time_ms != candle.open_time_ms
            {
                let closed = last.clone();
                tracing::debug!(coin = ?closed.coin, interval = ?closed.interval, open_time = closed.open_time_ms, "Candle closed and added to buffer");
                self.engine.push_closed_candle(candle_key.clone(), closed);
            }
        }

        // adding new candle
        self.engine.set_last_seen(candle_key.clone(), candle);

        // Checking for breakout
        let Some(view) = self.engine.market_view(&candle_key) else
        {
            return;
        };

        for alert in self.event_evaluator.evaluate(&view)
        {
            match alert.event
            {
                Event::AtrBreakout { atr, live_tr, ratio, spike_level, open_time_ms } =>
                {
                    tracing::info!
                    (
                        coin = ?alert.key.coin,
                        interval = ?alert.key.interval,
                        open_time = open_time_ms,
                        atr = atr,
                        live_tr = live_tr,
                        ratio = ratio,
                        spike_level = spike_level,
                        "ATR breakout detected"
                    );
                }
            }
        }
    }
}
