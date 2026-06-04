use std::error::Error;

use crate::market_data::{
    clients::hyperliquid::hl_client::run_hyperliquid_client,
    runtime::MarketDataRuntime,
    types::CandleKey,
};

pub async fn run_market_data_clients(
    candle_keys: &[CandleKey],
    runtime: &mut MarketDataRuntime,
) -> Result<(), Box<dyn Error>> {
    run_hyperliquid_client(candle_keys, runtime).await
}
