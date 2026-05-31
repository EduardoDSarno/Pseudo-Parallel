use std::collections::HashMap;

use crate::market_data::{
    config::MarketDataConfig,
    engine::Engine,
    hyperliquid::protocols::rest::RestResponse,
    signal::{evaluate::event_evaluator::EventEvaluator, price::PriceAlertService},
    types::{CandleKey, Coins},
};

/* This file is the composition root for the market data engine.
It owns all the pieces (engine, alert service, evaluators) in one place.
The coordinator orchestrator holds the playbook (process) but does not own state —
main and hl_client hold a MarketDataRuntime and call into it. */

/* MarketDataRuntime is the single object that wires everything together.
Engine stores candles, alert_service stores price levels, event_evaluator runs
price and indicator checks. last_market_price_by_coin tracks coin-level price
for crossing detection (not per timeframe). */
pub struct MarketDataRuntime {
    pub engine: Engine,
    alert_service: PriceAlertService,
    pub(crate) event_evaluator: EventEvaluator,
    pub(crate) last_market_price_by_coin: HashMap<Coins, f64>,
    config: MarketDataConfig,
}

impl MarketDataRuntime {
    pub fn new(config: MarketDataConfig) -> Self {
        MarketDataRuntime {
            engine: Engine::new(config.max_closed_candles),
            alert_service: PriceAlertService::new(),
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

    pub(crate) fn last_market_price(&self, coin: Coins) -> Option<f64> {
        self.last_market_price_by_coin.get(&coin).copied()
    }

    pub(crate) fn set_last_market_price(&mut self, coin: Coins, price: f64) -> Option<f64> {
        self.last_market_price_by_coin.insert(coin, price)
    }
}
