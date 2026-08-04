use std::{
    collections::BTreeMap,
    ops::Bound::{Excluded, Included},
};

use crate::market_data::signal::price::{
    key::{ManualPriceDirection, PriceKey},
    price_book::entry::PriceLevelEntry,
};

/* deivides in 2 ,maps for quicker search */
#[derive(Debug, Clone, Default)]
pub struct CoinPriceBook {
    above_map: BTreeMap<PriceKey, PriceLevelEntry>,
    below_map: BTreeMap<PriceKey, PriceLevelEntry>,
}

impl CoinPriceBook {
    fn get_map_mut(
        &mut self,
        direction: ManualPriceDirection,
    ) -> &mut BTreeMap<PriceKey, PriceLevelEntry> {
        match direction {
            ManualPriceDirection::Above => &mut self.above_map,
            ManualPriceDirection::Below => &mut self.below_map,
        }
    }

    /* If there's a entry increment subscribers otherwise add the entry */
    pub fn insert_level(
        &mut self,
        price_key: PriceKey,
        direction: ManualPriceDirection,
        entry: PriceLevelEntry,
    ) {
        let map = self.get_map_mut(direction);
        map.entry(price_key)
            .and_modify(|existing| existing.add_subscriber())
            .or_insert(entry);
    }

    /* if there's a entry remove subscriber count if none left remove entry, otherwise none */
    pub fn remove_level(
        &mut self,
        price_key: PriceKey,
        direction: ManualPriceDirection,
    ) -> Option<PriceLevelEntry> {
        let map = self.get_map_mut(direction);
        let should_remove = match map.get_mut(&price_key) {
            Some(entry) => entry.remove_subscriber(),
            None => return None,
        };

        if should_remove {
            return map.remove(&price_key);
        }

        map.get(&price_key).cloned()
    }

    /* This function will delete the whole level of alerts in pricebooks  */
    pub fn delete_level(
        &mut self,
        price_key: PriceKey,
        direction: ManualPriceDirection,
    ) -> Option<PriceLevelEntry> {
        self.get_map_mut(direction).remove(&price_key)
    }

    /* This fucntion returns a empty vec if none found otherwise it will return a vec with the prices crossed abvoe */
    pub fn levels_crossed_above(
        &self,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<(PriceKey, &PriceLevelEntry)> {
        if current_price <= previous_price {
            return Vec::new();
        }

        // getting keys
        let Some(previous_key) = PriceKey::from_price(previous_price) else {
            return Vec::new();
        };
        let Some(current_key) = PriceKey::from_price(current_price) else {
            return Vec::new();
        };

        self.above_map
            // shorten the range of search
            .range((Excluded(previous_key), Included(current_key)))
            .map(|(key, entry)| (*key, entry))
            .collect()
    }

    /* All currently active levels in this coin's book, both directions — for
    inspection/lookup, not used by the crossing-check path. */
    pub fn active_levels(&self) -> Vec<(ManualPriceDirection, PriceKey, &PriceLevelEntry)> {
        self.above_map
            .iter()
            .map(|(price_key, entry)| (ManualPriceDirection::Above, *price_key, entry))
            .chain(
                self.below_map
                    .iter()
                    .map(|(price_key, entry)| (ManualPriceDirection::Below, *price_key, entry)),
            )
            .collect()
    }

    /* This fucntion returns a empty vec if none found otherwise it will return a vec with the prices crossed below */
    pub fn levels_crossed_below(
        &self,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<(PriceKey, &PriceLevelEntry)> {
        if current_price >= previous_price {
            return Vec::new();
        }

        let Some(previous_key) = PriceKey::from_price(previous_price) else {
            return Vec::new();
        };
        let Some(current_key) = PriceKey::from_price(current_price) else {
            return Vec::new();
        };

        self.below_map
            .range((Included(current_key), Excluded(previous_key)))
            .map(|(key, entry)| (*key, entry))
            .collect()
    }
}
