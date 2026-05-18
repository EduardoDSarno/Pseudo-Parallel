use super::{AlertBook, ManualPriceAlert, ManualPriceDirection};
use crate::market_data::types::Coins;

const TEST_TRIGGER_PRICE: f64 = 42.0;

fn alert(direction: ManualPriceDirection) -> ManualPriceAlert {
    ManualPriceAlert::new(Coins::HYPE, TEST_TRIGGER_PRICE, direction)
}

#[test]
fn duplicate_insert_increments_subscriber_count() {
    let mut book = AlertBook::new();
    let key = book
        .insert_alert(alert(ManualPriceDirection::Above))
        .unwrap();
    let reused_key = book
        .insert_alert(alert(ManualPriceDirection::Above))
        .unwrap();

    assert_eq!(key, reused_key);
    assert_eq!(book._subscriber_count(&key), Some(2));
}

#[test]
fn delete_decrements_before_removing_shared_alert() {
    let mut book = AlertBook::new();
    let key = book
        .insert_alert(alert(ManualPriceDirection::Above))
        .unwrap();
    book.insert_alert(alert(ManualPriceDirection::Above))
        .unwrap();

    let deleted = book._delete_alert(key).unwrap();

    assert_eq!(deleted.coin, Coins::HYPE);
    assert!(book._contains_key(&key));
    assert_eq!(book._subscriber_count(&key), Some(1));

    book._delete_alert(key).unwrap();

    assert!(!book._contains_key(&key));
    assert_eq!(book._subscriber_count(&key), None);
}

#[test]
fn same_price_with_different_direction_is_different_alert() {
    let mut book = AlertBook::new();
    let above_key = book
        .insert_alert(alert(ManualPriceDirection::Above))
        .unwrap();
    let below_key = book
        .insert_alert(alert(ManualPriceDirection::Below))
        .unwrap();

    assert_ne!(above_key, below_key);
    assert_eq!(book._subscriber_count(&above_key), Some(1));
    assert_eq!(book._subscriber_count(&below_key), Some(1));
}

#[test]
fn crossed_above_returns_alerts_in_range() {
    let mut book = AlertBook::new();
    book.insert_alert(ManualPriceAlert::new(
        Coins::HYPE,
        41.0,
        ManualPriceDirection::Above,
    ))
    .unwrap();
    book.insert_alert(ManualPriceAlert::new(
        Coins::HYPE,
        43.0,
        ManualPriceDirection::Above,
    ))
    .unwrap();

    let alerts = book.alerts_crossed_above(Coins::HYPE, 40.0, 42.0);

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].trigger_price, 41.0);
}

#[test]
fn crossed_below_returns_alerts_in_range() {
    let mut book = AlertBook::new();
    book.insert_alert(ManualPriceAlert::new(
        Coins::HYPE,
        41.0,
        ManualPriceDirection::Below,
    ))
    .unwrap();
    book.insert_alert(ManualPriceAlert::new(
        Coins::HYPE,
        39.0,
        ManualPriceDirection::Below,
    ))
    .unwrap();

    let alerts = book.alerts_crossed_below(Coins::HYPE, 42.0, 40.0);

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].trigger_price, 41.0);
}

#[test]
fn no_crossing_returns_empty_alerts() {
    let mut book = AlertBook::new();
    book.insert_alert(alert(ManualPriceDirection::Above))
        .unwrap();
    book.insert_alert(alert(ManualPriceDirection::Below))
        .unwrap();

    assert!(book
        .alerts_crossed_above(Coins::HYPE, 43.0, 41.0)
        .is_empty());
    assert!(book
        .alerts_crossed_below(Coins::HYPE, 41.0, 43.0)
        .is_empty());
}
