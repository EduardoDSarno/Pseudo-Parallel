use std::{
    collections::{BTreeMap, HashMap},
    time::SystemTime,
};

use hypersdk::{
    Address, Decimal,
    hypercore::{AssetPosition, ClearinghouseState, PositionData},
};
use tokio::sync::{mpsc::Receiver, watch};

use crate::{
    coin::Coin,
    config::{LIQUIDATION_BUCKET_SIZE_USD, hyperliquid_time_to_system_time},
    heatmap::HeatmapSnapshot,
    liquidation::LiquidationLevel,
    price_data::CurrentPrice,
};

/// A position that passed the requirements for whale monitoring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilteredPosition {
    pub address: Address,
    pub coin: Coin,
    pub signed_size: Decimal,
    pub position_usd: Decimal,
    pub liquidation_price: Decimal,
    pub updated_at: SystemTime,
}

/// The result of successfully checking an account. `None` means that the
/// account no longer has a position that qualifies for whale monitoring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PositionUpdate {
    pub address: Address,
    pub position: Option<FilteredPosition>,
}

/// Owns both representations of the current whale data. `whale_positions` is
/// the source of truth, while `liquidation_levels` is updated from each change.
struct WhalePositionTracker {
    whale_positions: HashMap<Address, FilteredPosition>,
    liquidation_levels: BTreeMap<Decimal, LiquidationLevel>,
    /// price distance covered by each liquidation level (config)
    bucket_size: Decimal,
}

impl WhalePositionTracker {
    fn new(bucket_size: Decimal) -> Self {
        assert!(bucket_size > Decimal::ZERO, "bucket size must be positive");

        Self {
            whale_positions: HashMap::new(),
            liquidation_levels: BTreeMap::new(),
            bucket_size,
        }
    }

    /// Applies one authoritative account result to both maps. Keeping this in
    /// one method prevents the positions and liquidation levels from drifting.
    fn apply(&mut self, update: PositionUpdate) {
        match update.position {
            // add or update the wallet
            Some(position) => {
                // Calculate the new contribution before moving the position
                // into the HashMap.
                let new_bucket = self.bucket_price(position.liquidation_price);
                let new_estimated_usd = Self::estimated_liquidation_usd(&position);

                // HashMap::insert returns the position previously stored for
                // this address, so we know which old level must be decreased.
                let previous = self.whale_positions.insert(update.address, position);

                if let Some(previous) = previous {
                    self.remove_from_level(&previous);
                }

                self.add_to_level(new_bucket, new_estimated_usd);
            }
            // remove the wallet if previously stored
            // meaning the new clearinghouse response says this wallet does not currently
            // have a qualifying whale position
            None => {
                // HashMap::remove gives us the old position needed to subtract
                // its exact contribution from the previous liquidation level.
                if let Some(previous) = self.whale_positions.remove(&update.address) {
                    self.remove_from_level(&previous);
                }
            }
        }
    }

    fn add_to_level(&mut self, bucket_price: Decimal, estimated_usd: Decimal) {
        // entry creates a new empty level only when this bucket does not exist.
        let level = self.liquidation_levels.entry(bucket_price).or_default();
        level.estimated_liquidation_usd += estimated_usd;
        level.position_count += 1;
    }

    fn remove_from_level(&mut self, position: &FilteredPosition) {
        let bucket_price = self.bucket_price(position.liquidation_price);
        let estimated_usd = Self::estimated_liquidation_usd(position);

        let should_remove_level =
            if let Some(level) = self.liquidation_levels.get_mut(&bucket_price) {
                // A stored position must already have one matching level entry.

                // checking count before decrementing to avoid
                // usize out of bounds
                debug_assert!(level.position_count > 0);
                debug_assert!(level.estimated_liquidation_usd >= estimated_usd);

                level.estimated_liquidation_usd -= estimated_usd;
                level.position_count -= 1;
                // returns true if there's no more elements in the bucket
                level.position_count == 0
            } else {
                debug_assert!(false, "stored position had no liquidation level");
                false
            };

        // Remove empty buckets instead of leaving zero-value levels in the map.
        if should_remove_level {
            self.liquidation_levels.remove(&bucket_price);
        }
    }

    /// returns the bucket a position will go bases on liquidation price
    fn bucket_price(&self, liquidation_price: Decimal) -> Decimal {
        (liquidation_price / self.bucket_size).floor() * self.bucket_size
    }

    /// returns amount of usd that will be liquidated on the position
    fn estimated_liquidation_usd(position: &FilteredPosition) -> Decimal {
        position.signed_size.abs() * position.liquidation_price
    }

    /// Builds and prints the current heatmap only after the first market price
    /// has arrived. The underlying maps remain owned by this tracker.
    fn display_heatmap(&self, current_price: Option<CurrentPrice>) {
        let Some(current_price) = current_price else {
            return;
        };

        let Some(snapshot) = HeatmapSnapshot::build(
            current_price.coin,
            current_price.mark_price,
            &self.liquidation_levels,
            SystemTime::now(),
        ) else {
            return;
        };

        println!("Heatmap snapshot: {snapshot:#?}");
    }
}

/// Receives filtered position updates and keeps the whale position and
/// liquidation-level maps alive for the lifetime of this task.
pub async fn run_position_tracker(
    mut position_update_rx: Receiver<PositionUpdate>,
    mut current_price_rx: watch::Receiver<Option<CurrentPrice>>,
) {
    let mut tracker = WhalePositionTracker::new(LIQUIDATION_BUCKET_SIZE_USD);
    let mut price_channel_open = true;

    loop {
        // Wait for either kind of input without blocking the other task.
        tokio::select! {
            update = position_update_rx.recv() => {
                let Some(update) = update else {
                    break;
                };

                tracker.apply(update);

                // borrow_and_update also marks this price as already seen, so
                // select does not rebuild twice for the same price version.
                let current_price = *current_price_rx.borrow_and_update();
                tracker.display_heatmap(current_price);
            }
            price_change = current_price_rx.changed(), if price_channel_open => {
                match price_change {
                    Ok(()) => {
                        let current_price = *current_price_rx.borrow_and_update();
                        tracker.display_heatmap(current_price);
                    }
                    Err(_) => {
                        // Continue consuming remaining positions if the market
                        // price producer closes before the position channel.
                        price_channel_open = false;
                    }
                }
            }
        }
    }
}

/// Returns position data for the requested coin.
fn get_position_by_coin(states: &[AssetPosition], coin: Coin) -> Option<&PositionData> {
    states
        .iter()
        .find(|state| state.position.coin == coin.as_hyperliquid_symbol())
        .map(|state| &state.position)
}

/// Finds the requested position and transforms it only when it is useful for
/// whale-liquidation monitoring.
pub fn filter_whale_position(
    address: Address,
    state: &ClearinghouseState,
    coin: Coin,
    minimum_position_usd: Decimal,
) -> Option<FilteredPosition> {
    if address == Address::ZERO || minimum_position_usd < Decimal::ZERO {
        return None;
    }

    let position = get_position_by_coin(&state.asset_positions, coin)?;
    let position_usd = position.position_value.abs();
    let liquidation_price = position.liquidation_px?;

    if position.szi == Decimal::ZERO
        || position_usd < minimum_position_usd
        || liquidation_price <= Decimal::ZERO
    {
        return None;
    }

    Some(FilteredPosition {
        address,
        coin,
        signed_size: position.szi,
        position_usd,
        liquidation_price,
        updated_at: hyperliquid_time_to_system_time(state.time)?,
    })
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use hypersdk::{
        Address, Decimal, dec,
        hypercore::{
            AssetPosition, ClearinghouseState, CumulativeFunding, Leverage, LeverageType,
            MarginSummary, PositionData, PositionType,
        },
    };

    use super::{FilteredPosition, PositionUpdate, WhalePositionTracker, filter_whale_position};
    use crate::coin::Coin;

    fn clearinghouse_state(
        coin: &str,
        signed_size: Decimal,
        position_usd: Decimal,
        liquidation_price: Option<Decimal>,
    ) -> ClearinghouseState {
        let empty_summary = || MarginSummary {
            account_value: Decimal::ZERO,
            total_ntl_pos: Decimal::ZERO,
            total_raw_usd: Decimal::ZERO,
            total_margin_used: Decimal::ZERO,
        };

        ClearinghouseState {
            margin_summary: empty_summary(),
            cross_margin_summary: empty_summary(),
            cross_maintenance_margin_used: Decimal::ZERO,
            withdrawable: Decimal::ZERO,
            asset_positions: vec![AssetPosition {
                position_type: PositionType::OneWay,
                position: PositionData {
                    coin: coin.to_owned(),
                    szi: signed_size,
                    leverage: Leverage {
                        leverage_type: LeverageType::Cross,
                        value: 20,
                        raw_usd: None,
                    },
                    entry_px: Some(dec!(100_000)),
                    position_value: position_usd,
                    unrealized_pnl: Decimal::ZERO,
                    return_on_equity: Decimal::ZERO,
                    liquidation_px: liquidation_price,
                    margin_used: Decimal::ZERO,
                    max_leverage: 40,
                    cum_funding: CumulativeFunding {
                        all_time: Decimal::ZERO,
                        since_open: Decimal::ZERO,
                        since_change: Decimal::ZERO,
                    },
                },
            }],
            time: 1_754_345_600_123,
        }
    }

    fn address(value: &str) -> Address {
        value.parse().expect("test address should be valid")
    }

    fn filtered_position(
        address: Address,
        signed_size: Decimal,
        liquidation_price: Decimal,
    ) -> FilteredPosition {
        FilteredPosition {
            address,
            coin: Coin::Btc,
            signed_size,
            position_usd: signed_size.abs() * dec!(100_000),
            liquidation_price,
            updated_at: UNIX_EPOCH,
        }
    }

    fn update(position: FilteredPosition) -> PositionUpdate {
        PositionUpdate {
            address: position.address,
            position: Some(position),
        }
    }

    #[test]
    fn transforms_a_qualifying_whale_position() {
        let state = clearinghouse_state("BTC", dec!(-12.5), dec!(1_250_000), Some(dec!(105_000)));

        let position = filter_whale_position(
            address("0x1111111111111111111111111111111111111111"),
            &state,
            Coin::Btc,
            dec!(1_000_000),
        )
        .expect("position should qualify");

        assert_eq!(position.coin, Coin::Btc);
        assert_eq!(position.signed_size, dec!(-12.5));
        assert_eq!(position.position_usd, dec!(1_250_000));
        assert_eq!(position.liquidation_price, dec!(105_000));
    }

    #[test]
    fn rejects_a_position_below_the_whale_threshold() {
        let state = clearinghouse_state("BTC", dec!(5), dec!(999_999), Some(dec!(90_000)));

        assert!(
            filter_whale_position(
                address("0x1111111111111111111111111111111111111111"),
                &state,
                Coin::Btc,
                dec!(1_000_000),
            )
            .is_none()
        );
    }

    #[test]
    fn rejects_a_position_without_a_liquidation_price() {
        let state = clearinghouse_state("BTC", dec!(20), dec!(2_000_000), None);

        assert!(
            filter_whale_position(
                address("0x1111111111111111111111111111111111111111"),
                &state,
                Coin::Btc,
                dec!(1_000_000),
            )
            .is_none()
        );
    }

    #[test]
    fn combines_positions_inside_the_same_liquidation_bucket() {
        let mut tracker = WhalePositionTracker::new(dec!(100));

        tracker.apply(update(filtered_position(
            address("0x1111111111111111111111111111111111111111"),
            dec!(10),
            dec!(95_120),
        )));
        tracker.apply(update(filtered_position(
            address("0x2222222222222222222222222222222222222222"),
            dec!(-5),
            dec!(95_180),
        )));

        let level = tracker
            .liquidation_levels
            .get(&dec!(95_100))
            .expect("combined level should exist");

        assert_eq!(tracker.whale_positions.len(), 2);
        assert_eq!(level.position_count, 2);
        assert_eq!(level.estimated_liquidation_usd, dec!(1_427_100));
    }

    #[test]
    fn moving_a_position_updates_only_its_old_and_new_levels() {
        let mut tracker = WhalePositionTracker::new(dec!(100));

        let alice = address("0x1111111111111111111111111111111111111111");
        tracker.apply(update(filtered_position(alice, dec!(10), dec!(95_120))));
        tracker.apply(update(filtered_position(alice, dec!(12), dec!(96_020))));

        assert!(!tracker.liquidation_levels.contains_key(&dec!(95_100)));

        let level = tracker
            .liquidation_levels
            .get(&dec!(96_000))
            .expect("new level should exist");
        assert_eq!(tracker.whale_positions.len(), 1);
        assert_eq!(level.position_count, 1);
        assert_eq!(level.estimated_liquidation_usd, dec!(1_152_240));
    }

    #[test]
    fn removing_a_position_removes_its_empty_level() {
        let mut tracker = WhalePositionTracker::new(dec!(100));
        let alice = address("0x1111111111111111111111111111111111111111");
        tracker.apply(update(filtered_position(alice, dec!(10), dec!(95_120))));

        tracker.apply(PositionUpdate {
            address: alice,
            position: None,
        });

        assert!(tracker.whale_positions.is_empty());
        assert!(tracker.liquidation_levels.is_empty());
    }
}
