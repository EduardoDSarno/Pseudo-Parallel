use std::collections::{BTreeMap, HashMap};

use crate::market_data::signal::price_alert::alert::{AlertKey, ManualPriceAlert, ManualPriceDirection, PriceKey};
use crate::market_data::types::Coins;
use crate::Error;

/* This file is reposible for the creation of the data structure of mine Manual Alert books
    THe flow is the follow:
    The top of the Cake layer we have AlertBook which will map each of the Books By coin
    Giving it's IndividualBook then inside each Indivual Book we divided between above and 
    below represeting alerts that are to be triggered if the price cross above or below the
    threshold. Inside each map we use a BTree to store self balacing individual alerts, based 
    on the PriceKey which links to The individual Alert*/



/* Per-coin alert storage: above/below maps ordered by trigger price */
#[derive(Debug, Clone, Default)]
pub struct IndividualPriceAlertBook
{
    above_map: BTreeMap<PriceKey, ManualPriceAlert>,
    below_map: BTreeMap<PriceKey, ManualPriceAlert>,
}

impl IndividualPriceAlertBook
{
    /* Match direction */
    fn map_mut(&mut self, direction: ManualPriceDirection) -> &mut BTreeMap<PriceKey, ManualPriceAlert>
    {
        match direction 
        {
            ManualPriceDirection::Above => &mut self.above_map,
            ManualPriceDirection::Below => &mut self.below_map,
        }
    }

    pub fn get(&self, price_key: PriceKey, direction: ManualPriceDirection) -> Option<&ManualPriceAlert>
    {
        match direction 
        {
            ManualPriceDirection::Above => self.above_map.get(&price_key),
            ManualPriceDirection::Below => self.below_map.get(&price_key),
        }
    }

    /* Insert if inexistent */
    pub fn insert(&mut self, price_key: PriceKey, direction: ManualPriceDirection, alert: ManualPriceAlert) -> Result<(), String>
    {
        let map = self.map_mut(direction);
        if map.contains_key(&price_key)
        {
            return Err(format!("alert already exists at this price level ({:?}, {:?})", price_key, direction));
        }
        map.insert(price_key, alert);
        Ok(())
    }

    /* Remove with no Check because bTree remove already does that */
    pub fn remove(&mut self, price_key: PriceKey, direction: ManualPriceDirection) -> Option<ManualPriceAlert>
    {
        self.map_mut(direction).remove(&price_key)
    }
}

/* Routes by coin; each IndividualPriceAlertBook holds that coin's alerts */
pub struct AlertBook
{
    books: HashMap<Coins, IndividualPriceAlertBook>,
}

impl AlertBook
{
    pub fn new() -> Self
    {
        AlertBook 
        {
            books: HashMap::new(),
        }
    }

    pub fn insert_alert(&mut self, alert: ManualPriceAlert) -> Result<AlertKey, Box<dyn Error>>
    {
        let key = alert.alert_key()?;
        let book = self.book_for_coin(key.coin);

        if let Some(existing) = book.get(key.price_key, key.direction) 
        {
            tracing::debug!(?key, "alert already exists, reusing key");
            return existing.alert_key();
        }

        book.insert(key.price_key, key.direction, alert)
            .map_err(|e| -> Box<dyn Error> { e.into() })?;

        tracing::debug!(?key, coin = ?key.coin, "alert inserted");
        Ok(key)
    }

    pub fn delete_alert(&mut self, key: AlertKey) -> Result<ManualPriceAlert, Box<dyn Error>>
    {
        match self.books.get_mut(&key.coin) 
        {
            Some(book) => book.remove(key.price_key, key.direction)
                .ok_or_else(|| -> Box<dyn Error> { "alert not found".into() }),
            None => Err("alert not found".into()),
        }
    }

    pub fn get_alert(&self, key: &AlertKey) -> Option<&ManualPriceAlert>
    {
        match self.books.get(&key.coin) 
        {
            Some(book) => book.get(key.price_key, key.direction),
            None => None,
        }
    }

    /* HELPERS */

    pub fn contains_key(&self, key: &AlertKey) -> bool
    {
        self.get_alert(key).is_some()
    }

    pub fn book_for_coin(&mut self, coin: Coins) -> &mut IndividualPriceAlertBook
    {
        self.books.entry(coin).or_default()
    }

    pub fn book_for_coin_ref(&self, coin: Coins) -> Option<&IndividualPriceAlertBook>
    {
        self.books.get(&coin)
    }
}
