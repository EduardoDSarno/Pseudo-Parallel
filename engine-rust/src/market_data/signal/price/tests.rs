use super::{AlertBook, ManualPriceAlert, ManualPriceDirection};
use crate::market_data::types::Coins;

const TEST_TRIGGER_PRICE: f64 = 42.0;

fn alert(direction: ManualPriceDirection) -> ManualPriceAlert
{
    ManualPriceAlert::new(Coins::HYPE, TEST_TRIGGER_PRICE, direction)
}

#[test]
fn duplicate_insert_increments_subscriber_count()
{
    let mut book = AlertBook::new();
    let key = book.insert_alert(alert(ManualPriceDirection::Above)).unwrap();
    let reused_key = book.insert_alert(alert(ManualPriceDirection::Above)).unwrap();

    assert_eq!(key, reused_key);
    assert_eq!(book.subscriber_count(&key), Some(2));
}

#[test]
fn delete_decrements_before_removing_shared_alert()
{
    let mut book = AlertBook::new();
    let key = book.insert_alert(alert(ManualPriceDirection::Above)).unwrap();
    book.insert_alert(alert(ManualPriceDirection::Above)).unwrap();

    let deleted = book.delete_alert(key).unwrap();

    assert_eq!(deleted.coin, Coins::HYPE);
    assert!(book.contains_key(&key));
    assert_eq!(book.subscriber_count(&key), Some(1));

    book.delete_alert(key).unwrap();

    assert!(!book.contains_key(&key));
    assert_eq!(book.subscriber_count(&key), None);
}

#[test]
fn same_price_with_different_direction_is_different_alert()
{
    let mut book = AlertBook::new();
    let above_key = book.insert_alert(alert(ManualPriceDirection::Above)).unwrap();
    let below_key = book.insert_alert(alert(ManualPriceDirection::Below)).unwrap();

    assert_ne!(above_key, below_key);
    assert_eq!(book.subscriber_count(&above_key), Some(1));
    assert_eq!(book.subscriber_count(&below_key), Some(1));
}
