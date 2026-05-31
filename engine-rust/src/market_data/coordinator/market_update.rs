use crate::market_data::types::Candle;

/* Expandable market updates types */
pub enum MarketUpdate {
    Candle(Candle),
}
