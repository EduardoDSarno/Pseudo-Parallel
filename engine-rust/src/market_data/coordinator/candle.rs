use crate::market_data::{
    runtime::MarketDataRuntime,
    signal::event::{Alert, Event},
    types::{Candle, CandleKey, Coins},
};

impl MarketDataRuntime {
    pub fn handle_candle(&mut self, candle: Candle) {
        let candle_key = CandleKey::create_key_from_candle(&candle);

        if let Some(last) = self.engine.last_seen(&candle_key) {
            if last.open_time_ms != candle.open_time_ms {
                let closed = last.clone();
                tracing::debug!(coin = ?closed.coin, interval = ?closed.interval, open_time = closed.open_time_ms, "Candle closed and added to buffer");
                self.engine.push_closed_candle(candle_key.clone(), closed);
            }
        }

        self.engine.set_last_seen(candle_key.clone(), candle.clone());

        let coin = candle.coin;
        let current_price = candle.close_price;

        let mut alerts = self.price_alerts_if_coin_price_changed(coin, current_price);

        if let Some(view) = self.engine.market_view(&candle_key) {
            alerts.extend(self.event_evaluator.evaluate_indicators(&view));
        }

        for alert in alerts {
            match alert.event {
                Event::AtrBreakout {
                    atr,
                    live_tr,
                    ratio,
                    spike_level,
                    open_time_ms,
                } => {
                    let key = alert.key.expect("ATR alerts always carry a candle key");
                    tracing::info!(
                        coin = ?alert.coin,
                        interval = ?key.interval,
                        open_time = open_time_ms,
                        atr = atr,
                        live_tr = live_tr,
                        ratio = ratio,
                        spike_level = spike_level,
                        "ATR breakout detected"
                    );
                }
                Event::ManualPriceTriggered {
                    trigger_price,
                    direction,
                    previous_price,
                    current_price,
                } => {
                    tracing::info!(
                        coin = ?alert.coin,
                        trigger_price = trigger_price,
                        direction = ?direction,
                        previous_price = previous_price,
                        current_price = current_price,
                        "Manual price alert triggered"
                    );
                }
            }
        }
    }

    fn price_alerts_if_coin_price_changed(&mut self, coin: Coins, current_price: f64) -> Vec<Alert> {
        let Some(&previous_price) = self.last_market_price_by_coin.get(&coin) else {
            self.last_market_price_by_coin.insert(coin, current_price);
            return Vec::new();
        };

        if previous_price == current_price {
            return Vec::new();
        }

        self.last_market_price_by_coin
            .insert(coin, current_price);

        self.event_evaluator.evaluate_price(
            self.alert_service(),
            coin,
            previous_price,
            current_price,
        )
    }
}
