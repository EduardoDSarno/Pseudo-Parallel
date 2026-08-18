use hypersdk::Decimal;

/// Aggregated whale exposure inside one liquidation-price bucket.
/// The bucket price itself is stored as the BTreeMap key.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LiquidationLevel {
    /// Estimated position value at the liquidation price.
    pub estimated_liquidation_usd: Decimal,
    /// Number of whale positions contributing to this level.
    pub position_count: usize,
}
