use std::{collections::HashMap, error::Error};

use crate::market_data::{
    config::MarketDataConfig,
    engine::Engine,
    hyperliquid::protocols::rest::RestResponse,
    signal::{
        evaluate::event_evaluator::EventEvaluator, indicator_rules::IndicatorRuleService,
        price::PriceAlertService,
    },
    subscriptions::command::{SubscriptionCommand, SubscriptionManager, SubscriptionType},
    types::{CandleKey, Coins},
};

/* This file is the composition root for the market data engine.
It owns all the pieces (engine, alert service, evaluators) in one place.
The coordinator orchestrator holds the playbook (process) but does not own state —
main and hl_client hold a MarketDataRuntime and call into it. */

/* MarketDataRuntime is the single object that wires everything together.
Engine stores candles, alert_service stores price levels, event_evaluator runs
price and indicator checks. indicator_rule_service stores active indicator rules.
last_market_price_by_coin tracks coin-level price
for crossing detection (not per timeframe). */
pub struct MarketDataRuntime {
    pub engine: Engine,
    alert_service: PriceAlertService,
    indicator_rule_service: IndicatorRuleService,
    pub(crate) event_evaluator: EventEvaluator,
    pub(crate) last_market_price_by_coin: HashMap<Coins, f64>,
    config: MarketDataConfig,
}

impl MarketDataRuntime {
    pub fn new(config: MarketDataConfig) -> Self {
        MarketDataRuntime {
            engine: Engine::new(config.max_closed_candles),
            alert_service: PriceAlertService::new(),
            indicator_rule_service: IndicatorRuleService::new(),
            event_evaluator: EventEvaluator::new(config.max_closed_candles),
            last_market_price_by_coin: HashMap::new(),
            config,
        }
    }

    pub fn max_closed_candles(&self) -> usize {
        self.config.max_closed_candles
    }

    /* Wrapper so startup can seed the engine without touching engine directly */
    pub fn seed_from_rest_responses(
        &mut self,
        responses: Vec<RestResponse>,
        seed_end_time: u64,
    ) -> Result<(), String> {
        self.engine
            .seed_from_rest_responses(responses, seed_end_time)
    }

    pub fn verify_seeded_keys(&self, keys: &[CandleKey]) -> Result<(), String> {
        self.engine.verify_seeded_keys(keys)
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

    pub fn apply_subscription(&mut self, sub: &SubscriptionManager) -> Result<(), Box<dyn Error>> {
        match sub.command {
            SubscriptionCommand::Subscribe => match &sub.sub_type {
                SubscriptionType::Price(alert) => {
                    self.alert_service_mut().subscribe(alert.clone())?;
                }
                SubscriptionType::Indicator(ind) => {
                    self.indicator_rule_service_mut()
                        .subscribe(ind.key.clone(), ind.kind.clone());
                }
            },
            SubscriptionCommand::Unsubscribe => match &sub.sub_type {
                SubscriptionType::Price(alert) => {
                    let key = alert.alert_key().map_err(|err| {
                        tracing::error!(
                            error = %err,
                            ?alert,
                            "apply_subscription: invalid price alert key for unsubscribe"
                        );
                        err
                    })?;
                    self.alert_service_mut().unsubscribe(key).map_err(|err| {
                        tracing::warn!(
                            error = %err,
                            ?alert,
                            "apply_subscription: price alert unsubscribe failed"
                        );
                        err
                    })?;
                }
                SubscriptionType::Indicator(ind) => {
                    self.indicator_rule_service_mut()
                        .unsubscribe(ind.key.clone(), ind.kind.clone())
                        .map_err(|err| {
                            tracing::warn!(
                                error = %err,
                                key = ?ind.key,
                                "apply_subscription: indicator rule unsubscribe failed"
                            );
                            err
                        })?;
                }
            },
        }
        Ok(())
    }

    pub fn load_signal_subscriptions(
        &mut self,
        subs: Vec<SubscriptionManager>,
    ) -> Result<(), Box<dyn Error>> {
        for sub in subs {
            self.apply_subscription(&sub)?;
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
