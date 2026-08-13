use hypersdk::Decimal;

/// Input messages for the market data consumer.
#[derive(Debug)]
pub enum MarketInput {
    PriceUpdate { coin: String, mark_price: Decimal },
}
