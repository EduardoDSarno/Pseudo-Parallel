/* Defines the exact JSON shape sent through Redis when a price alert fires. */

use serde::Serialize;

use crate::market_data::{signal::price::ManualPriceDirection, types::Coins};

#[derive(Serialize)]
pub struct OutgoingManualPriceAlert {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub coin: Coins,
    pub trigger_price: f64,
    pub direction: ManualPriceDirection,
    pub current_price: f64,
}

impl OutgoingManualPriceAlert {
    pub fn new(
        coin: Coins,
        trigger_price: f64,
        direction: ManualPriceDirection,
        current_price: f64,
    ) -> Self {
        OutgoingManualPriceAlert {
            kind: "manual_price",
            coin,
            trigger_price,
            direction,
            current_price,
        }
    }

    pub fn convert(self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}
