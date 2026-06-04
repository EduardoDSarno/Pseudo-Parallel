/* Pure price-alert direction resolution — no runtime / store access. */

use std::error::Error;

use crate::market_data::{
    signal::price::{alert::ManualPriceAlert, ManualPriceDirection},
    types::Coins,
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

/* Caller supplies reference_price when direction is omitted (e.g. from apply + runtime). */
pub fn build_manual_price_alert(
    coin: Coins,
    trigger_price: f64,
    direction: Option<ManualPriceDirection>,
    reference_price: Option<f64>,
) -> Result<ManualPriceAlert, Box<dyn Error>> {
    let direction = match direction {
        Some(d) => d,
        None => {
            let reference = reference_price.ok_or_else(|| {
                format!("no reference price for {coin:?}; cannot infer direction")
            })?;
            resolve_price_direction(reference, trigger_price)?
        }
    };
    Ok(ManualPriceAlert::new(coin, trigger_price, direction))
}
