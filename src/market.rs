use std::{fmt, time::SystemTime};

use chrono::{DateTime, Local};
use hypersdk::Decimal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Coin {
    Btc,
}

impl Coin {

    /// Returns the Hyperliquid symbol for the coin.
    /// In other words, it just converts our enum into a static string.
    pub fn as_hyperliquid_symbol(self) -> &'static str {
        match self {
            Self::Btc => "BTC",
        }
    }
}

/// Display the coin as the Hyperliquid symbol.
/// This is useful for logging and debugging.
impl fmt::Display for Coin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_hyperliquid_symbol())
    }
}

/// Input messages for the market data consumer.
/// Explicitly represents a market price update.
#[derive(Debug)]
pub enum MarketInput {
    PriceUpdate {
        coin: Coin,
        mark_price: Decimal,
        timestamp: SystemTime,
    },
}

impl MarketInput 
{
    pub fn display(self) -> ()
    {
        match self 
        {
            MarketInput::PriceUpdate { coin, mark_price, timestamp } =>
            {
                let local_time: DateTime<Local> = timestamp.into();
                println!(
                    "{coin} mark price: {mark_price} at {}",
                    local_time.format("%Y-%m-%d %H:%M:%S%.3f")
                );
            },
        }
       
    }
}
