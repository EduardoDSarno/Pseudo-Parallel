use std::collections::HashMap;

use crate::market_data::{
    signal::price::{
        key::{ManualPriceDirection, PriceKey},
        price_book::{coin_book::CoinPriceBook, entry::PriceLevelEntry},
    },
    types::Coins,
};

/* Price books is our top level api for the boopks it holds per each 
    coin their coins price Books (separated above and below) */
pub struct PriceBook {
    by_coin: HashMap<Coins, CoinPriceBook>,
}

impl PriceBook {
    pub fn new() -> Self {
        PriceBook {
            by_coin: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        coin: Coins,
        price_key: PriceKey,
        direction: ManualPriceDirection,
        entry: PriceLevelEntry,
    ) {
        self.coin_book_mut(coin)
            .insert_level(price_key, direction, entry);
    }

    pub fn remove(
        &mut self,
        coin: Coins,
        price_key: PriceKey,
        direction: ManualPriceDirection,
    ) -> Option<PriceLevelEntry> {
        self.by_coin
            .get_mut(&coin)
            .and_then(|book| book.remove_level(price_key, direction))
    }

    pub fn get(
        &self,
        coin: Coins,
        price_key: PriceKey,
        direction: ManualPriceDirection,
    ) -> Option<&PriceLevelEntry> {
        self.by_coin
            .get(&coin)
            .and_then(|book| book.get_level(price_key, direction))
    }

    pub fn contains(
        &self,
        coin: Coins,
        price_key: PriceKey,
        direction: ManualPriceDirection,
    ) -> bool {
        self.get(coin, price_key, direction).is_some()
    }

    pub fn subscriber_count(
        &self,
        coin: Coins,
        price_key: PriceKey,
        direction: ManualPriceDirection,
    ) -> Option<usize> {
        self.get(coin, price_key, direction)
            .map(|entry| entry.subscriber_count())
    }

    /* levels corssed per coin */
    pub fn levels_crossed_above(
        &self,
        coin: Coins,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<(PriceKey, &PriceLevelEntry)> {
        self.by_coin
            .get(&coin)
            .map(|book| book.levels_crossed_above(previous_price, current_price))
            .unwrap_or_default()
    }

    pub fn levels_crossed_below(
        &self,
        coin: Coins,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<(PriceKey, &PriceLevelEntry)> {
        self.by_coin
            .get(&coin)
            .map(|book| book.levels_crossed_below(previous_price, current_price))
            .unwrap_or_default()
    }

    fn coin_book_mut(&mut self, coin: Coins) -> &mut CoinPriceBook {
        self.by_coin.entry(coin).or_default()
    }
}
