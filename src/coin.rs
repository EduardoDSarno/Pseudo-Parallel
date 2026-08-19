use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::Coin;

    #[test]
    fn btc_uses_hyperliquid_btc_symbol() {
        assert_eq!(Coin::Btc.as_hyperliquid_symbol(), "BTC");
        assert_eq!(Coin::Btc.to_string(), "BTC");
    }
}
