/* Resolve price alert direction at apply time (needs runtime / market price). */

use std::error::Error;

use crate::market_data::{
    runtime::MarketDataRuntime,
    signal::price::{alert::ManualPriceAlert, ManualPriceDirection},
    types::{CandleKey, Coins, Interval},
};

/* Above if market is below trigger; Below if market is above trigger. */
pub fn resolve_price_direction(
    reference: f64,
    trigger: f64,
) -> Result<ManualPriceDirection, Box<dyn Error>> {
    if reference < trigger {
        Ok(ManualPriceDirection::Above)
    } else if reference > trigger {
        Ok(ManualPriceDirection::Below)
    } else {
        Err(format!(
            "reference price {reference} equals trigger {trigger}; cannot infer direction"
        )
        .into())
    }
}

/* Same source as price crossing: last M5 tick price, else latest M5 candle close. */
pub fn reference_price_for_coin(runtime: &MarketDataRuntime, coin: Coins) -> Option<f64> {
    if let Some(price) = runtime.last_market_price(coin) {
        return Some(price);
    }
    let key = CandleKey::new(coin, Interval::M5);
    let view = runtime.candle_store.market_view(&key)?;
    view
        .closed_candles
        .back()
        .map(|c| c.close_price)
        .or(Some(view.live_candle.close_price))
}

pub fn build_manual_price_alert(
    runtime: &MarketDataRuntime,
    coin: Coins,
    trigger_price: f64,
    direction: Option<ManualPriceDirection>,
) -> Result<ManualPriceAlert, Box<dyn Error>> {
    let direction = match direction {
        Some(d) => d,
        None => {
            let reference = reference_price_for_coin(runtime, coin).ok_or_else(|| {
                format!("no reference price yet for {coin:?}; cannot infer direction")
            })?;
            resolve_price_direction(reference, trigger_price)?
        }
    };
    Ok(ManualPriceAlert::new(coin, trigger_price, direction))
}
