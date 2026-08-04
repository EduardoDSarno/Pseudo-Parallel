use crate::market_data::{
    signal::price::{alert::ManualPriceAlert, ManualPriceDirection, PriceAlertService},
    types::Coins,
};

const TEST_TRIGGER_PRICE: f64 = 42.0;

fn alert(direction: ManualPriceDirection) -> ManualPriceAlert {
    ManualPriceAlert::new(Coins::HYPE, TEST_TRIGGER_PRICE, direction)
}

#[test]
fn duplicate_subscriptions_require_two_unsubscribes() {
    let mut service = PriceAlertService::new();
    let key = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    let reused_key = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();

    assert_eq!(key, reused_key);
    assert!(service.unsubscribe(key).is_ok());
    assert!(service.unsubscribe(key).is_ok());
    assert!(service.unsubscribe(key).is_err());
}

#[test]
fn same_price_with_different_directions_uses_different_levels() {
    let mut service = PriceAlertService::new();
    let above = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    let below = service
        .subscribe(alert(ManualPriceDirection::Below))
        .unwrap();

    assert_ne!(above, below);
    assert!(service.unsubscribe(above).is_ok());
    assert!(service.unsubscribe(below).is_ok());
}

#[test]
fn upward_cross_returns_and_removes_level() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();

    let first = service.take_crossed(Coins::HYPE, 40.0, 43.0);
    let second = service.take_crossed(Coins::HYPE, 40.0, 43.0);

    assert_eq!(first.len(), 1);
    assert_eq!(first[0].trigger_price, TEST_TRIGGER_PRICE);
    assert_eq!(first[0].current_price, 43.0);
    assert!(second.is_empty());
}

#[test]
fn downward_cross_returns_and_removes_level() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(alert(ManualPriceDirection::Below))
        .unwrap();

    let alerts = service.take_crossed(Coins::HYPE, 43.0, 41.0);

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].trigger_price, TEST_TRIGGER_PRICE);
    assert_eq!(alerts[0].direction, ManualPriceDirection::Below);
}

#[test]
fn crossing_removes_shared_level_once() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();

    assert_eq!(service.take_crossed(Coins::HYPE, 40.0, 43.0).len(), 1);
    assert!(service.take_crossed(Coins::HYPE, 40.0, 43.0).is_empty());
}

#[test]
fn movement_without_crossing_returns_no_alerts() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();

    assert!(service.take_crossed(Coins::HYPE, 39.0, 41.0).is_empty());
    assert!(service.take_crossed(Coins::HYPE, 41.0, 41.0).is_empty());
}

#[test]
fn active_alerts_for_coin_is_empty_when_none_subscribed() {
    let service = PriceAlertService::new();
    assert!(service.active_alerts_for_coin(Coins::HYPE).is_empty());
}

#[test]
fn active_alerts_for_coin_lists_current_subscriptions() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    service
        .subscribe(ManualPriceAlert::new(Coins::HYPE, 50.0, ManualPriceDirection::Below))
        .unwrap();

    let mut active = service.active_alerts_for_coin(Coins::HYPE);
    active.sort_by(|a, b| a.trigger_price.partial_cmp(&b.trigger_price).unwrap());

    assert_eq!(active.len(), 2);
    assert_eq!(active[0].trigger_price, TEST_TRIGGER_PRICE);
    assert_eq!(active[0].direction, ManualPriceDirection::Above);
    assert_eq!(active[1].trigger_price, 50.0);
    assert_eq!(active[1].direction, ManualPriceDirection::Below);
}

#[test]
fn unsubscribing_removes_alert_from_active_list() {
    let mut service = PriceAlertService::new();
    let key = service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();

    service.unsubscribe(key).unwrap();

    assert!(service.active_alerts_for_coin(Coins::HYPE).is_empty());
}

#[test]
fn all_active_alerts_groups_by_coin() {
    let mut service = PriceAlertService::new();
    service
        .subscribe(alert(ManualPriceDirection::Above))
        .unwrap();
    service
        .subscribe(ManualPriceAlert::new(
            Coins::BTC,
            60_000.0,
            ManualPriceDirection::Above,
        ))
        .unwrap();

    let all = service.all_active_alerts();

    assert_eq!(all.get(&Coins::HYPE).map(Vec::len), Some(1));
    assert_eq!(all.get(&Coins::BTC).map(Vec::len), Some(1));
    assert!(!all.contains_key(&Coins::ETH));
}
