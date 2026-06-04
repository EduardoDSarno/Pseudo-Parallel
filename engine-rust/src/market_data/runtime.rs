use std::{collections::HashMap, error::Error};

use crate::market_data::{
    candle_store::CandleStore,
    clients::hyperliquid::protocols::rest::RestResponse,
    config::MarketDataConfig,
    signal::{
        evaluate::event_evaluator::EventEvaluator, indicator_rules::IndicatorRuleService,
        price::PriceAlertService,
    },
    subscriptions::{
        apply::apply_subscription,
        command::SubscriptionManager,
    },
    types::{CandleKey, Coins},
};

/* This file is the composition root for the market data runtime.
It owns all the pieces (candle store, alert service, evaluators) in one place.
The coordinator orchestrator holds the playbook (process) but does not own state —
main and hl_client hold a MarketDataRuntime and call into it. */

/* MarketDataRuntime is the single object that wires everything together.
CandleStore stores candles, alert_service stores price levels, event_evaluator runs
price and indicator checks. indicator_rule_service stores active indicator rules.
last_market_price_by_coin tracks coin-level price
for crossing detection (not per timeframe). */
pub struct MarketDataRuntime {
    pub candle_store: CandleStore,
    alert_service: PriceAlertService,
    indicator_rule_service: IndicatorRuleService,
    pub(crate) event_evaluator: EventEvaluator,
    pub(crate) last_market_price_by_coin: HashMap<Coins, f64>,
    config: MarketDataConfig,
}

impl MarketDataRuntime {
    pub fn new(config: MarketDataConfig) -> Self {
        MarketDataRuntime {
            candle_store: CandleStore::new(config.max_closed_candles),
            alert_service: PriceAlertService::new(),
            indicator_rule_service: IndicatorRuleService::new(),
            event_evaluator: EventEvaluator::new(),
            last_market_price_by_coin: HashMap::new(),
            config,
        }
    }

    pub fn max_closed_candles(&self) -> usize {
        self.config.max_closed_candles
    }

    /* Wrapper so startup can seed candles without touching the candle store directly */
    pub fn seed_from_rest_responses(
        &mut self,
        responses: Vec<RestResponse>,
        seed_end_time: u64,
    ) -> Result<(), String> {
        self.candle_store
            .seed_from_rest_responses(responses, seed_end_time)
    }

    pub fn verify_seeded_keys(&self, keys: &[CandleKey]) -> Result<(), String> {
        self.candle_store.verify_seeded_keys(keys)
    }

    /* Read-only access for price evaluation during process */
    pub fn alert_service(&self) -> &PriceAlertService {
        &self.alert_service
    }

    /* Mutable access for the future subscription stream to subscribe/unsubscribe */
    pub fn alert_service_mut(&mut self) -> &mut PriceAlertService {
        &mut self.alert_service
    }

    pub fn indicator_rule_service(&self) -> &IndicatorRuleService {
        &self.indicator_rule_service
    }

    pub fn indicator_rule_service_mut(&mut self) -> &mut IndicatorRuleService {
        &mut self.indicator_rule_service
    }

    #[cfg(test)]
    pub fn load_default_indicator_rules(&mut self, keys: &[CandleKey]) {
        self.indicator_rule_service.subscribe_default_atr_rules(
            keys,
            self.config.default_atr_breakout_ratio,
            self.config.default_live_atr_debug_ratio,
        );
    }

    pub fn load_signal_subscriptions(
        &mut self,
        subs: Vec<SubscriptionManager>,
    ) -> Result<(), Box<dyn Error>> {
        for sub in subs {
            apply_subscription(self, &sub)?;
        }
        Ok(())
    }

    pub(crate) fn last_market_price(&self, coin: Coins) -> Option<f64> {
        self.last_market_price_by_coin.get(&coin).copied()
    }

    pub(crate) fn set_last_market_price(&mut self, coin: Coins, price: f64) -> Option<f64> {
        self.last_market_price_by_coin.insert(coin, price)
    }
}
