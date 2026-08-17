use std::{fmt, time::SystemTime};

use chrono::{DateTime, Local};
use hypersdk::{Address, Decimal};

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
    TradeObserved {
        coin: Coin,
        buyer: String,
        seller: String,
        timestamp: SystemTime,
    },
}

impl MarketInput {
    /// Creates a mark-price update when the external source supplied a price.
    /// Missing prices are rejected at the ingestion boundary.
    pub(crate) fn create_mark_price_update(
        coin: Coin,
        mark_price: Option<Decimal>,
        timestamp: SystemTime,
    ) -> Option<Self> {
        mark_price.map(|mark_price| Self::PriceUpdate {
            coin,
            mark_price,
            timestamp,
        })
    }

    /// Creates a trade-observed event when the exchange supplied real trade participants.
    /// Trades reported without buyer/seller addresses are rejected at the ingestion boundary.
    pub(crate) fn create_trade_record(
        coin: Coin,
        buyer: Address,
        seller: Address,
        timestamp: SystemTime,
    ) -> Option<Self> {
        if buyer == Address::ZERO || seller == Address::ZERO {
            return None;
        }

        Some(Self::TradeObserved {
            coin,
            buyer: buyer.to_string(),
            seller: seller.to_string(),
            timestamp,
        })
    }

    /// Prints the market input in a human-readable form.
    pub fn display(&self) {
        match self {
            MarketInput::PriceUpdate {
                coin,
                mark_price,
                timestamp,
            } => {
                let local_time: DateTime<Local> = (*timestamp).into();
                println!(
                    "{coin} mark price: {mark_price} at {}",
                    local_time.format("%Y-%m-%d %H:%M:%S%.3f")
                );
            }
            MarketInput::TradeObserved {
                coin,
                buyer,
                seller,
                timestamp,
            } => {
                let local_time: DateTime<Local> = (*timestamp).into();
                println!(
                    "{coin} trade buyer: {buyer} seller: {seller} at {}",
                    local_time.format("%Y-%m-%d %H:%M:%S%.3f")
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use hypersdk::{Address, Decimal};

    use super::{Coin, MarketInput};

    #[test]
    fn btc_uses_hyperliquid_btc_symbol() {
        assert_eq!(Coin::Btc.as_hyperliquid_symbol(), "BTC");
        assert_eq!(Coin::Btc.to_string(), "BTC");
    }

    #[test]
    fn valid_mark_price_creates_price_update() {
        let price = Decimal::new(6_500_025, 2);
        let timestamp = UNIX_EPOCH + Duration::from_millis(1_750_000_000_123);

        let input = MarketInput::create_mark_price_update(Coin::Btc, Some(price), timestamp);

        match input {
            Some(MarketInput::PriceUpdate {
                coin,
                mark_price,
                timestamp: observed_at,
            }) => {
                assert_eq!(coin, Coin::Btc);
                assert_eq!(mark_price, price);
                assert_eq!(observed_at, timestamp);
            }
            _ => panic!("a valid mark price should create a market input"),
        }
    }

    #[test]
    fn missing_mark_price_does_not_create_price_update() {
        let input = MarketInput::create_mark_price_update(Coin::Btc, None, UNIX_EPOCH);

        assert!(input.is_none());
    }

    #[test]
    fn valid_trade_creates_trade_observed() {
        let buyer: Address = "0x1111111111111111111111111111111111111111"
            .parse()
            .unwrap();
        let seller: Address = "0x2222222222222222222222222222222222222222"
            .parse()
            .unwrap();
        let timestamp = UNIX_EPOCH + Duration::from_millis(1_750_000_000_123);

        let input = MarketInput::create_trade_record(Coin::Btc, buyer, seller, timestamp);

        match input {
            Some(MarketInput::TradeObserved {
                coin,
                buyer: observed_buyer,
                seller: observed_seller,
                timestamp: observed_at,
            }) => {
                assert_eq!(coin, Coin::Btc);
                assert_eq!(observed_buyer, buyer.to_string());
                assert_eq!(observed_seller, seller.to_string());
                assert_eq!(observed_at, timestamp);
            }
            _ => panic!("a trade with real participants should create a market input"),
        }
    }

    #[test]
    fn missing_participants_does_not_create_trade_observed() {
        let input =
            MarketInput::create_trade_record(Coin::Btc, Address::ZERO, Address::ZERO, UNIX_EPOCH);

        assert!(input.is_none());
    }
}
