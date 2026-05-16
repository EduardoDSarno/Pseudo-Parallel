use std::collections::BTreeMap;

use crate::market_data::{constans::{PRICE_SCALE}, types::{Candle, Coins}};
use crate::Error;



pub struct ManualPriceAlert 
{
    pub coin: Coins,
    pub trigger_price: f64,
    pub direction: ManualPriceDirection,
    pub active: bool,
}

// For float conversion Since Btree do not work with float due to NaN values
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PriceKey(pub i64);

/* THis struct is for defining our pricealert books on which the 2 different maps 
    are just to represent the alers set to below or above a certain price range */
pub struct ManualPriceAlertBook
{
    above_map: BTreeMap<PriceKey, Vec<ManualPriceAlert>>,
    below_map: BTreeMap<PriceKey, Vec<ManualPriceAlert>>,
}
pub struct ManualPriceAlertEvent 
{
    pub coin: Coins,
    pub trigger_price: i64,
    pub current_price: i64,
    pub direction: ManualPriceDirection,
    pub timestamp: u64
}

pub enum ManualPriceDirection 
{
    Above,
    Below,
}

impl ManualPriceAlert
{
    fn new(coin: Coins, trigger_p: f64, direction: ManualPriceDirection, active:bool) -> Self
    {
        let alert = ManualPriceAlert
        {
            coin:coin,
            trigger_price: trigger_p,
            direction: direction,
            active: active
        };
        alert
    }
}

impl ManualPriceAlertBook
{
    pub fn new() ->Self
    {

        let map_above: BTreeMap<PriceKey, Vec<ManualPriceAlert>> = BTreeMap::new();
        let map_below: BTreeMap<PriceKey, Vec<ManualPriceAlert>> = BTreeMap::new();

        let book = ManualPriceAlertBook
        {
            above_map: map_above,
            below_map: map_below
        };
        book
    }

    pub fn insert_alert(&mut self, alert: ManualPriceAlert) -> Result<(), Box<dyn Error>>
    {
        let p_key = price_to_key(alert.trigger_price)
                              .ok_or("invalid trigger price")?;

        match alert.direction 
        {
            // If key exits get the vec if doenst insert a new vec, then pus alert
            ManualPriceDirection::Above => 
            {
                self.above_map
                .entry(p_key)
                .or_insert_with(Vec::new)
                .push(alert);
            }
            ManualPriceDirection::Below =>
            {
                self.below_map
                .entry(p_key)
                .or_insert_with(Vec::new)
                .push(alert);
            }    
        }
        Ok(())
    }
}

pub fn price_to_key(price: f64) -> Option<PriceKey>
{
    if !price.is_finite() || price <= 0.0
    {
        return None;
    }

    Some(PriceKey((price * PRICE_SCALE).round() as i64))
}