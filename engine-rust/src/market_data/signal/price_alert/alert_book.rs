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
    on the PriceKey which links to The individual Alert

    The design was made to be a realtionship one to many between users and alerts where multiple users
    can point to the same alert insated of creating duplications
    
    The wrapper of ManualPriceAlertEntry was added for the integreation of users and Alerts
    since the map needs to know how many entries it has for a Alert to track deletion*/



/* Per-coin alert storage: above/below maps ordered by trigger price */
#[derive(Debug, Clone, Default)]
pub struct IndividualPriceAlertBook
{
    above_map: BTreeMap<PriceKey, ManualPriceAlertEntry>,
    below_map: BTreeMap<PriceKey, ManualPriceAlertEntry>,
}

/* Internal storage record for one shared alert rule */
#[derive(Debug, Clone)]
pub struct ManualPriceAlertEntry
{
    alert: ManualPriceAlert,
    subscriber_count: usize,
}

impl ManualPriceAlertEntry
{
    /* Creates a new entry for one shared alert with one subscriber */
    fn new(alert: ManualPriceAlert) -> Self
    {
        ManualPriceAlertEntry
        {
            alert: alert,
            subscriber_count: 1,
        }
    }

    /* Adds one more subscriber to this shared rule */
    fn add_subscriber(&mut self)
    {
        self.subscriber_count += 1;
    }

    /* Decrements and Return if still subscriber */
    fn remove_subscriber(&mut self) -> bool
    {
        self.subscriber_count -= 1;
        self.subscriber_count == 0
    }

    /* Returns the alert definition stored in this entry */
    pub fn alert(&self) -> &ManualPriceAlert
    {
        &self.alert
    }

    /* Returns how many subscribers point at this rule */
    pub fn subscriber_count(&self) -> usize
    {
        self.subscriber_count
    }
}

impl IndividualPriceAlertBook
{
    /* Match direction */
    fn map_mut(&mut self, direction: ManualPriceDirection) -> &mut BTreeMap<PriceKey, ManualPriceAlertEntry>
    {
        match direction 
        {
            ManualPriceDirection::Above => &mut self.above_map,
            ManualPriceDirection::Below => &mut self.below_map,
        }
    }

    /* Returns the above or below map for reading */
    fn map_ref(&self, direction: ManualPriceDirection) -> &BTreeMap<PriceKey, ManualPriceAlertEntry>
    {
        match direction
        {
            ManualPriceDirection::Above => &self.above_map,
            ManualPriceDirection::Below => &self.below_map,
        }
    }

    /* Looks up an alert by price level and direction */
    pub fn get(&self, price_key: PriceKey, direction: ManualPriceDirection) -> Option<&ManualPriceAlert>
    {
        self.map_ref(direction)
            .get(&price_key)
            .map(|entry| entry.alert())
    }

    /* Looks up the full entry by price level and direction */
    pub fn get_entry(&self, price_key: PriceKey, direction: ManualPriceDirection) -> Option<&ManualPriceAlertEntry>
    {
        self.map_ref(direction).get(&price_key)
    }

    /* Insert if inexistent; otherwise add one more subscriber to the shared rule */
    pub fn insert(&mut self, price_key: PriceKey, direction: ManualPriceDirection, alert: ManualPriceAlert)
    {
        let map = self.map_mut(direction);
        map.entry(price_key)
            .and_modify(|entry| entry.add_subscriber())
            .or_insert_with(|| ManualPriceAlertEntry::new(alert));
    }

    /* Remove one subscriber. Delete the shared rule only when nobody points to it anymore */
    pub fn remove(&mut self, price_key: PriceKey, direction: ManualPriceDirection) -> Option<ManualPriceAlert>
    {
        let map = self.map_mut(direction);
        let should_remove = match map.get_mut(&price_key)
        {
            Some(entry) => entry.remove_subscriber(),
            None => return None,
        };

        if should_remove
        {
            return map.remove(&price_key).map(|entry| entry.alert);
        }

        map.get(&price_key).map(|entry| entry.alert.clone())
    }
}

/* Routes by coin; each IndividualPriceAlertBook holds that coin's alerts */
pub struct AlertBook
{
    books: HashMap<Coins, IndividualPriceAlertBook>,
}

impl AlertBook
{
    /* Creates an empty alert book */
    pub fn new() -> Self
    {
        AlertBook 
        {
            books: HashMap::new(),
        }
    }

    /* Inserts alert or adds a subscriber if the rule already exists */
    pub fn insert_alert(&mut self, alert: ManualPriceAlert) -> Result<AlertKey, Box<dyn Error>>
    {
        let key = alert.alert_key()?;
        let book = self.book_for_coin(key.coin);

        book.insert(key.price_key, key.direction, alert);

        tracing::debug!(?key, coin = ?key.coin, "alert inserted or subscriber added");
        Ok(key)
    }

    /* Removes one subscriber; deletes the rule when nobody points to it */
    pub fn delete_alert(&mut self, key: AlertKey) -> Result<ManualPriceAlert, Box<dyn Error>>
    {
        match self.books.get_mut(&key.coin) 
        {
            Some(book) => book.remove(key.price_key, key.direction)
                .ok_or_else(|| -> Box<dyn Error> { "alert not found".into() }),
            None => Err("alert not found".into()),
        }
    }

    /* Returns the alert for a key if it exists */
    pub fn get_alert(&self, key: &AlertKey) -> Option<&ManualPriceAlert>
    {
        match self.books.get(&key.coin) 
        {
            Some(book) => book.get(key.price_key, key.direction),
            None => None,
        }
    }

    /* HELPERS */

    /* Returns true if this alert key exists */
    pub fn contains_key(&self, key: &AlertKey) -> bool
    {
        self.get_alert(key).is_some()
    }

    /* Returns how many subscribers share this alert key */
    pub fn subscriber_count(&self, key: &AlertKey) -> Option<usize>
    {
        self.books
            .get(&key.coin)
            .and_then(|book| book.get_entry(key.price_key, key.direction))
            .map(|entry| entry.subscriber_count())
    }

    /* Gets or creates the alert book for a coin */
    pub fn book_for_coin(&mut self, coin: Coins) -> &mut IndividualPriceAlertBook
    {
        self.books.entry(coin).or_default()
    }

    /* Returns the alert book for a coin without creating it */
    pub fn book_for_coin_ref(&self, coin: Coins) -> Option<&IndividualPriceAlertBook>
    {
        self.books.get(&coin)
    }
}
