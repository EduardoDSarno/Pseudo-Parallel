use crate::market_data::{constans::PRICE_SCALE, types::Coins};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManualPriceDirection 
{
    Above,
    Below,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PriceKey(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelKey {
    pub coin: Coins,
    pub price_key: PriceKey,
    pub direction: ManualPriceDirection,
}

/* Wrap price key in struct to avoid Nan */
impl PriceKey {
    pub fn from_price(price: f64) -> Option<PriceKey> {
        if !price.is_finite() || price <= 0.0 {
            return None;
        }

        Some(PriceKey((price * PRICE_SCALE).round() as i64))
    }
}

impl LevelKey {
    pub fn new(coin: Coins, price_key: PriceKey, direction: ManualPriceDirection) -> Self {
        LevelKey {
            coin,
            price_key,
            direction,
        }
    }
}
