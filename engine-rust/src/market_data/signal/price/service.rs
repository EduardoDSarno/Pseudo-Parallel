use std::collections::HashMap;
use std::error::Error;

use crate::market_data::{
    signal::price::{
        alert::{AlertKey, ManualPriceAlert, TriggeredPriceAlert},
        key::ManualPriceDirection,
        price_book::{entry::PriceLevelEntry, PriceBook},
    },
    types::Coins,
};

/* Alert service is the connector between our books and alerts it own books to keep it private*/
pub struct PriceAlertService {
    book: PriceBook,
}

impl PriceAlertService {
    pub fn new() -> Self {
        PriceAlertService {
            book: PriceBook::new(),
        }
    }

    /* Create a subscription entry on the books */
    pub fn subscribe(&mut self, alert: ManualPriceAlert) -> Result<AlertKey, Box<dyn Error>> {
        let key = alert.alert_key()?;
        let entry = PriceLevelEntry::new(alert.trigger_price);

        self.book
            .insert(key.coin, key.price_key, key.direction, entry);

        tracing::debug!(?key, coin = ?key.coin, "alert subscribed or subscriber added");
        Ok(key)
    }

    /* Delete subcription from books, return none if not found */
    pub fn unsubscribe(&mut self, key: AlertKey) -> Result<ManualPriceAlert, Box<dyn Error>> {
        let entry = self
            .book
            .remove(key.coin, key.price_key, key.direction)
            .ok_or_else(|| -> Box<dyn Error> { "alert not found".into() })?;

        tracing::debug!(?key, coin = ?key.coin, "alert unsubscribed");
        Ok(ManualPriceAlert::from_level(
            key.coin,
            key.direction,
            &entry,
        ))
    }

    /* Active alerts for one coin, as user-facing values — for lookup/debugging,
    not used by the crossing-check path. */
    pub fn active_alerts_for_coin(&self, coin: Coins) -> Vec<ManualPriceAlert> {
        self.book
            .active_levels_for_coin(coin)
            .into_iter()
            .map(|(direction, _price_key, entry)| ManualPriceAlert::from_level(coin, direction, entry))
            .collect()
    }

    /* Every active alert, grouped by coin. */
    pub fn all_active_alerts(&self) -> HashMap<Coins, Vec<ManualPriceAlert>> {
        self.book
            .active_levels()
            .into_iter()
            .map(|(coin, levels)| {
                let alerts = levels
                    .into_iter()
                    .map(|(direction, _price_key, entry)| {
                        ManualPriceAlert::from_level(coin, direction, entry)
                    })
                    .collect();
                (coin, alerts)
            })
            .collect()
    }

    /* Return crossed levels and remove them from the active book in one operation */
    pub fn take_crossed(
        &mut self,
        coin: Coins,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<TriggeredPriceAlert> {
        let direction = if current_price > previous_price {
            ManualPriceDirection::Above
        } else if current_price < previous_price {
            ManualPriceDirection::Below
        } else {
            return Vec::new();
        };

        let crossed = match direction {
            ManualPriceDirection::Above => {
                self.book
                    .levels_crossed_above(coin, previous_price, current_price)
            }
            ManualPriceDirection::Below => {
                self.book
                    .levels_crossed_below(coin, previous_price, current_price)
            }
        }
        .into_iter()
        .map(|(price_key, entry)| (price_key, entry.trigger_price))
        .collect::<Vec<_>>();

        crossed
            .into_iter()
            .map(|(price_key, trigger_price)| {
                self.book.delete_level(coin, price_key, direction);
                TriggeredPriceAlert {
                    coin,
                    trigger_price,
                    direction,
                    current_price,
                }
            })
            .collect()
    }
}
