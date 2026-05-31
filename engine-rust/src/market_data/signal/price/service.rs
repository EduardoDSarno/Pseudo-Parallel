use std::error::Error;

use crate::market_data::{
    signal::price::{
        alert::{AlertKey, ManualPriceAlert},
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

        Ok(ManualPriceAlert::from_level(
            key.coin,
            key.direction,
            &entry,
        ))
    }

    /* This function will delete the whole level of alerts in pricebooks  */
    pub fn disarm_level_on_trigger(&mut self, key: AlertKey) -> Option<PriceLevelEntry> {
        let removed = self
            .book
            .delete_level(key.coin, key.price_key, key.direction);

        if removed.is_some() {
            tracing::debug!(?key, coin = ?key.coin, "price level disarmed after trigger");
        }

        removed
    }

    pub fn disarm_levels(&mut self, keys: impl IntoIterator<Item = AlertKey>) {
        for key in keys {
            self.disarm_level_on_trigger(key);
        }
    }

    pub fn get(&self, key: &AlertKey) -> Option<ManualPriceAlert> {
        self.book
            .get(key.coin, key.price_key, key.direction)
            .map(|entry| ManualPriceAlert::from_level(key.coin, key.direction, entry))
    }

    pub fn contains(&self, key: &AlertKey) -> bool {
        self.book.contains(key.coin, key.price_key, key.direction)
    }

    pub fn subscriber_count(&self, key: &AlertKey) -> Option<usize> {
        self.book
            .subscriber_count(key.coin, key.price_key, key.direction)
    }

    /* Return levels crossed above per coin */
    pub fn crossed_above(
        &self,
        coin: Coins,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<ManualPriceAlert> {
        self.book
            .levels_crossed_above(coin, previous_price, current_price)
            .into_iter()
            .map(|(_, entry)| {
                ManualPriceAlert::from_level(coin, ManualPriceDirection::Above, entry)
            })
            .collect()
    }

    /* Return levels crossed below per coin */
    pub fn crossed_below(
        &self,
        coin: Coins,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<ManualPriceAlert> {
        self.book
            .levels_crossed_below(coin, previous_price, current_price)
            .into_iter()
            .map(|(_, entry)| {
                ManualPriceAlert::from_level(coin, ManualPriceDirection::Below, entry)
            })
            .collect()
    }
}
