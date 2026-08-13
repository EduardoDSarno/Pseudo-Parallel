use hypersdk::Decimal;

#[derive(Debug)]
pub enum MarketInput {
    PriceUpdate { coin: String, mark_price: Decimal },
}
