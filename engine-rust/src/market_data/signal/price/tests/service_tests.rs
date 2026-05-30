use crate::market_data::{
    signal::price::{PriceAlertService, ManualPriceAlert, ManualPriceDirection},
    types::Coins,
};

const TEST_TRIGGER_PRICE: f64 = 42.0;

fn alert(direction: ManualPriceDirection) -> ManualPriceAlert {
    ManualPriceAlert::new(Coins::HYPE, TEST_TRIGGER_PRICE, direction)
}

#[test]
fn duplicate_subscribe_increments_subscriber_count() {
    let mut service = PriceAlertService::new();
    let key = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    let reused_key = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();

    assert_eq!(key, reused_key);
    assert_eq!(service.subscriber_count(&key), Some(2));
}

#[test]
fn unsubscribe_decrements_before_removing_shared_alert() {
    let mut service = PriceAlertService::new();
    let key = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();

    let removed = service.unsubscribe(key).unwrap();

    assert_eq!(removed.coin, Coins::HYPE);
    assert!(service.contains(&key));
    assert_eq!(service.subscriber_count(&key), Some(1));

    service.unsubscribe(key).unwrap();

    assert!(!service.contains(&key));
    assert_eq!(service.subscriber_count(&key), None);
}

#[test]
fn same_price_with_different_direction_is_different_alert() {
    let mut service = PriceAlertService::new();
    let above_key = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    let below_key = service
        .subscribe(alert(ManualPriceDirection::Below))
        .unwrap();

    assert_ne!(above_key, below_key);
    assert_eq!(service.subscriber_count(&above_key), Some(1));
    assert_eq!(service.subscriber_count(&below_key), Some(1));
}

#[test]
fn crossed_above_returns_alerts_in_range() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(ManualPriceAlert::new(
            Coins::HYPE,
            41.0,
            ManualPriceDirection::Above,
        ))
        .unwrap();
    service
        .subscribe(ManualPriceAlert::new(
            Coins::HYPE,
            43.0,
            ManualPriceDirection::Above,
        ))
        .unwrap();

    let alerts = service.crossed_above(Coins::HYPE, 40.0, 42.0);

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].trigger_price, 41.0);
}

#[test]
fn crossed_below_returns_alerts_in_range() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(ManualPriceAlert::new(
            Coins::HYPE,
            41.0,
            ManualPriceDirection::Below,
        ))
        .unwrap();
    service
        .subscribe(ManualPriceAlert::new(
            Coins::HYPE,
            39.0,
            ManualPriceDirection::Below,
        ))
        .unwrap();

    let alerts = service.crossed_below(Coins::HYPE, 42.0, 40.0);

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].trigger_price, 41.0);
}

#[test]
fn no_crossing_returns_empty_alerts() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    service
        .subscribe(alert(ManualPriceDirection::Below))
        .unwrap();

    assert!(service
        .crossed_above(Coins::HYPE, 43.0, 41.0)
        .is_empty());
    assert!(service
        .crossed_below(Coins::HYPE, 41.0, 43.0)
        .is_empty());
}

#[test]
fn get_returns_subscribed_alert() {
    let mut service = PriceAlertService::new();
    let key = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();

    let alert = service.get(&key).unwrap();

    assert_eq!(alert.trigger_price, TEST_TRIGGER_PRICE);
    assert_eq!(alert.direction, ManualPriceDirection::Above);
}
