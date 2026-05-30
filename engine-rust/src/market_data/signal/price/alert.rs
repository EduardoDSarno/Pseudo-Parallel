#![allow(dead_code)]

use crate::market_data::{
    signal::price::{
        key::{LevelKey, ManualPriceDirection, PriceKey},
        price_book::entry::PriceLevelEntry,
    },
    types::Coins,
};
use crate::Error;

pub use crate::market_data::signal::price::key::{LevelKey as AlertKey}; // alaias for naming preference

/* The alert struct is suppose to the the user facing rule for the alers*/
#[derive(Debug, Clone)]
pub struct ManualPriceAlert {
    pub coin: Coins,
    pub trigger_price: f64,
    pub direction: ManualPriceDirection,
}

impl ManualPriceAlert {
    pub fn new(coin: Coins, trigger_p: f64, direction: ManualPriceDirection) -> Self {
        ManualPriceAlert {
            coin,
            trigger_price: trigger_p,
            direction,
        }
    }

    /* Maps then indivual alert from the books entry, for visualization */
    pub fn from_level(coin: Coins, direction: ManualPriceDirection, entry: &PriceLevelEntry) -> Self {
        ManualPriceAlert 
        {
            coin,
            trigger_price: entry.trigger_price,
            direction,
        }
    }

    pub fn alert_key(&self) -> Result<LevelKey, Box<dyn Error>>
    {
        let price_key = match PriceKey::from_price(self.trigger_price) {
            Some(key) => key,
            None => {
                tracing::error!(
                    trigger_price = self.trigger_price,
                    "invalid trigger price for alert key"
                );
                return Err("invalid trigger price".into());
            }
        };

        Ok(LevelKey::new(self.coin, price_key, self.direction))
    }
}
