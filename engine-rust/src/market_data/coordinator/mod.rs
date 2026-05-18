use crate::market_data::{
    config::MarketDataConfig,
    engine::Engine,
    hyperliquid::protocols::rest::RestResponse,
    signal::evaluate::event_evaluator::EventEvaluator,
};

mod candle;

pub struct MarketDataCoordinator
{
    pub(super) engine: Engine,
    pub(super) event_evaluator: EventEvaluator,
    config: MarketDataConfig,
}

impl MarketDataCoordinator
{
    pub fn new(config: MarketDataConfig) -> Self
    {
        let engine = Engine::new(config.max_closed_candles);
        let event_evaluator = EventEvaluator::new(config.max_closed_candles);

        MarketDataCoordinator 
        {
            engine,
            event_evaluator,
            config,
        }
    }

    /* Helpers */
    pub fn max_closed_candles(&self) -> usize
    {
        self.config.max_closed_candles
    }

    /*Wrap */
    pub fn seed_from_rest_responses(&mut self, responses: Vec<RestResponse>) -> Result<(), String>
    {
        self.engine.seed_from_rest_responses(responses)
    }
}
