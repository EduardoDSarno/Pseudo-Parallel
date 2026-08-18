use std::{collections::HashMap, time::SystemTime};

use hypersdk::{
    Decimal,
    hypercore::{AssetPosition, ClearinghouseState, PositionData},
};
use tokio::sync::mpsc::Receiver;

use crate::{config::hyperliquid_time_to_system_time, market::Coin};

/// A position that passed the requirements for whale monitoring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilteredPosition {
    pub address: String,
    pub coin: Coin,
    pub signed_size: Decimal,
    pub position_usd: Decimal,
    pub liquidation_price: Decimal,
    pub updated_at: SystemTime,
}

/// Requests an authoritative account lookup for one coin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountLookupRequest {
    pub address: String,
    pub coin: Coin,
}

/// The result of successfully checking an account. `None` means that the
/// account no longer has a position that qualifies for whale monitoring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PositionUpdate {
    pub address: String,
    pub position: Option<FilteredPosition>,
}

/// Temporarily displays position updates. This consumer will become the
/// whale-position tracker when persistent state is added.
pub async fn run_position_tracker(mut position_update_rx: Receiver<PositionUpdate>) {

    let mut whale_positions = HashMap::new();

    while let Some(update) = position_update_rx.recv().await {
        match update.position {
            Some(position) => 
            {
                whale_positions.insert(update.address, position);
            }
            None => {
                // removes it and returns Some(old_position) or changes nothing and returns None
                whale_positions.remove(&update.address);
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
    address: &str,
    state: &ClearinghouseState,
    coin: Coin,
    minimum_position_usd: Decimal,
) -> Option<FilteredPosition> {
    if address.is_empty() || minimum_position_usd < Decimal::ZERO {
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
        address: address.to_owned(),
        coin,
        signed_size: position.szi,
        position_usd,
        liquidation_price,
        updated_at: hyperliquid_time_to_system_time(state.time)?,
    })
}

#[cfg(test)]
mod tests {
    use hypersdk::{
        Decimal, dec,
        hypercore::{
            AssetPosition, ClearinghouseState, CumulativeFunding, Leverage, LeverageType,
            MarginSummary, PositionData, PositionType,
        },
    };

    use super::filter_whale_position;
    use crate::market::Coin;

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

    #[test]
    fn transforms_a_qualifying_whale_position() {
        let state = clearinghouse_state("BTC", dec!(-12.5), dec!(1_250_000), Some(dec!(105_000)));

        let position = filter_whale_position(
            "0x1111111111111111111111111111111111111111",
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

        assert!(filter_whale_position("wallet", &state, Coin::Btc, dec!(1_000_000)).is_none());
    }

    #[test]
    fn rejects_a_position_without_a_liquidation_price() {
        let state = clearinghouse_state("BTC", dec!(20), dec!(2_000_000), None);

        assert!(filter_whale_position("wallet", &state, Coin::Btc, dec!(1_000_000)).is_none());
    }
}
