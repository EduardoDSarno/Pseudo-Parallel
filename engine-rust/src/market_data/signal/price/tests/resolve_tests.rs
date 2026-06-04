use crate::market_data::{
    signal::price::{
        build_manual_price_alert,
        resolve::resolve_price_direction,
        ManualPriceDirection,
    },
    types::Coins,
};

#[test]
fn resolve_price_direction_above_when_market_below_trigger() {
    assert_eq!(
        resolve_price_direction(65.0, 70.0).unwrap(),
        ManualPriceDirection::Above
    );
}

#[test]
fn resolve_price_direction_below_when_market_above_trigger() {
    assert_eq!(
        resolve_price_direction(75.0, 70.0).unwrap(),
        ManualPriceDirection::Below
    );
}

#[test]
fn resolve_price_direction_rejects_equal_reference_and_trigger() {
    assert!(resolve_price_direction(70.0, 70.0).is_err());
}

#[test]
fn build_manual_price_alert_infers_from_reference() {
    let alert =
        build_manual_price_alert(Coins::HYPE, 70.0, None, Some(65.0)).unwrap();
    assert_eq!(alert.direction, ManualPriceDirection::Above);
}

#[test]
fn build_manual_price_alert_uses_explicit_direction_without_reference() {
    let alert = build_manual_price_alert(
        Coins::HYPE,
        70.0,
        Some(ManualPriceDirection::Below),
        None,
    )
    .unwrap();
    assert_eq!(alert.direction, ManualPriceDirection::Below);
}

#[test]
fn build_manual_price_alert_errors_when_direction_missing_and_no_reference() {
    assert!(build_manual_price_alert(Coins::HYPE, 70.0, None, None).is_err());
}
