use std::{collections::BTreeMap, time::SystemTime};

use hypersdk::{Decimal, dec};

use crate::market::Coin;

/// Percentage of the current price displayed on each side of the heatmap.
const HEATMAP_WINDOW_PERCENT: Decimal = dec!(5);
const ONE_HUNDRED_PERCENT: Decimal = dec!(100);

/// Aggregated whale exposure inside one liquidation-price bucket.
/// The bucket price itself is stored as the BTreeMap key.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LiquidationLevel {
    /// Estimated position value at the liquidation price.
    pub estimated_liquidation_usd: Decimal,
    /// Number of whale positions contributing to this level.
    pub position_count: usize,
}

/// One liquidation level prepared for displaying in the heatmap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeatmapLevel {
    pub price: Decimal,
    pub estimated_liquidation_usd: Decimal,
    pub position_count: usize,
    /// Signed distance from the current price. Levels below it are negative.
    pub distance_percent: Decimal,
}

/// A point-in-time view of the liquidation levels close to the market price.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeatmapSnapshot {
    pub coin: Coin,
    pub current_price: Decimal,
    pub lower_price: Decimal,
    pub upper_price: Decimal,
    pub levels: Vec<HeatmapLevel>,
    pub generated_at: SystemTime,
}

impl HeatmapSnapshot {
    /// Creates a heatmap view containing only levels within 5% below or above
    /// the current price. Returns `None` when the current price is not valid.
    pub fn build(
        coin: Coin,
        current_price: Decimal,
        liquidation_levels: &BTreeMap<Decimal, LiquidationLevel>,
        generated_at: SystemTime,
    ) -> Option<Self> {
        if current_price <= Decimal::ZERO {
            return None;
        }

        let window = current_price * HEATMAP_WINDOW_PERCENT / ONE_HUNDRED_PERCENT;
        let lower_price = current_price - window;
        let upper_price = current_price + window;

        // BTreeMap::range reads only the ordered levels inside our visible
        // price window instead of looping over every liquidation level.
        let levels = liquidation_levels
            .range(lower_price..=upper_price)
            .map(|(price, level)| HeatmapLevel {
                price: *price,
                estimated_liquidation_usd: level.estimated_liquidation_usd,
                position_count: level.position_count,
                distance_percent: (*price - current_price) / current_price * ONE_HUNDRED_PERCENT,
            })
            .collect();

        Some(Self {
            coin,
            current_price,
            lower_price,
            upper_price,
            levels,
            generated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, time::UNIX_EPOCH};

    use hypersdk::{Decimal, dec};

    use super::{HeatmapSnapshot, LiquidationLevel};
    use crate::market::Coin;

    fn level(estimated_liquidation_usd: Decimal) -> LiquidationLevel {
        LiquidationLevel {
            estimated_liquidation_usd,
            position_count: 1,
        }
    }

    #[test]
    fn builds_a_snapshot_with_only_levels_inside_the_five_percent_window() {
        let levels = BTreeMap::from([
            (dec!(94_900), level(dec!(1_000_000))),
            (dec!(95_000), level(dec!(2_000_000))),
            (dec!(100_000), level(dec!(3_000_000))),
            (dec!(105_000), level(dec!(4_000_000))),
            (dec!(105_100), level(dec!(5_000_000))),
        ]);

        let snapshot = HeatmapSnapshot::build(Coin::Btc, dec!(100_000), &levels, UNIX_EPOCH)
            .expect("a positive current price should create a snapshot");

        assert_eq!(snapshot.lower_price, dec!(95_000));
        assert_eq!(snapshot.upper_price, dec!(105_000));
        assert_eq!(snapshot.levels.len(), 3);
        assert_eq!(snapshot.levels[0].distance_percent, dec!(-5));
        assert_eq!(snapshot.levels[1].distance_percent, dec!(0));
        assert_eq!(snapshot.levels[2].distance_percent, dec!(5));
    }

    #[test]
    fn rejects_a_non_positive_current_price() {
        let levels = BTreeMap::new();

        assert!(HeatmapSnapshot::build(Coin::Btc, dec!(0), &levels, UNIX_EPOCH).is_none());
    }
}
