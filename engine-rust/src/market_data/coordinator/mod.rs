use crate::market_data::{
    config::MarketDataConfig,
    engine::Engine,
    hyperliquid::protocols::rest::RestResponse,
    signal::evaluate::evaluate_indicators::Evaluator,
};

mod candle;

pub struct MarketDataCoordinator
{
    pub(super) engine: Engine,
    pub(super) evaluator: Evaluator,
    config: MarketDataConfig,
}

impl MarketDataCoordinator
{
    pub fn new(config: MarketDataConfig) -> Self
    {
        let engine = Engine::new(config.max_closed_candles);
        let evaluator = Evaluator::new(config.max_closed_candles);

        MarketDataCoordinator 
        {
            engine,
            evaluator,
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
