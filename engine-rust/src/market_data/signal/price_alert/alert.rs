use crate::market_data::{constans::PRICE_SCALE, types::Coins};
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManualPriceDirection 
{
    Above,
    Below,
}

/* For float conversion since BTree does not work with float due to NaN values */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PriceKey(pub i64);

/* Full rule identity: coin + price level + direction */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlertKey 
{
    pub coin: Coins,
    pub price_key: PriceKey,
    pub direction: ManualPriceDirection,
}

/* The ALert Structure itself */
#[derive(Debug, Clone)]
pub struct ManualPriceAlert 
{
    pub coin: Coins,
    pub trigger_price: f64,
    pub direction: ManualPriceDirection,
}

impl ManualPriceAlert
{
    pub fn new(coin: Coins, trigger_p: f64, direction: ManualPriceDirection) -> Self
    {
        ManualPriceAlert
        {
            coin: coin,
            trigger_price: trigger_p,
            direction: direction,
        }
    }

    /*Returns a Result alert key for the ManualPriceALert*/
    pub fn alert_key(&self) -> Result<AlertKey, Box<dyn Error>> 
    {
        let price_key = match PriceKey::from_price(self.trigger_price) 
        {
            Some(key) => key,
            None => 
            {
                tracing::error!(trigger_price = self.trigger_price, "invalid trigger price for alert key");
                return Err("invalid trigger price".into());
            }
        };

        Ok(AlertKey::new(self.coin, price_key, self.direction))
    }
}

/* Price Key converter to go around f64 NAN limitation in search */
impl PriceKey
{
    pub fn from_price(price: f64) -> Option<PriceKey>
    {
        if !price.is_finite() || price <= 0.0
        {
            return None;
        }

        Some(PriceKey((price * PRICE_SCALE).round() as i64))
    }
}

impl AlertKey
{
    pub fn new(coin: Coins, price_key: PriceKey, direction: ManualPriceDirection) -> Self 
    {
        AlertKey 
        {
            coin: coin,
            price_key: price_key,
            direction: direction,
        }
    }
}
