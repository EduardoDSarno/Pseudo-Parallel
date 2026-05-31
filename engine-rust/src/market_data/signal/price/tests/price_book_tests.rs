use crate::market_data::{
    signal::price::{
        key::{ManualPriceDirection, PriceKey},
        price_book::{entry::PriceLevelEntry, PriceBook},
    },
    types::Coins,
};

const TEST_TRIGGER_PRICE: f64 = 42.0;

fn entry(trigger_price: f64) -> PriceLevelEntry {
    PriceLevelEntry::new(trigger_price)
}

fn price_key(trigger_price: f64) -> PriceKey {
    PriceKey::from_price(trigger_price).unwrap()
}

#[test]
fn duplicate_insert_increments_subscriber_count() {
    let mut book = PriceBook::new();
    let key = price_key(TEST_TRIGGER_PRICE);

    book.insert(
        Coins::HYPE,
        key,
        ManualPriceDirection::Above,
        entry(TEST_TRIGGER_PRICE),
    );
    book.insert(
        Coins::HYPE,
        key,
        ManualPriceDirection::Above,
        entry(TEST_TRIGGER_PRICE),
    );

    assert_eq!(
        book.subscriber_count(Coins::HYPE, key, ManualPriceDirection::Above),
        Some(2)
    );
}

#[test]
fn remove_decrements_before_removing_shared_level() {
    let mut book = PriceBook::new();
    let key = price_key(TEST_TRIGGER_PRICE);

    book.insert(
        Coins::HYPE,
        key,
        ManualPriceDirection::Above,
        entry(TEST_TRIGGER_PRICE),
    );
    book.insert(
        Coins::HYPE,
        key,
        ManualPriceDirection::Above,
        entry(TEST_TRIGGER_PRICE),
    );

    let removed = book
        .remove(Coins::HYPE, key, ManualPriceDirection::Above)
        .unwrap();

    assert_eq!(removed.trigger_price, TEST_TRIGGER_PRICE);
    assert!(book.contains(Coins::HYPE, key, ManualPriceDirection::Above));
    assert_eq!(
        book.subscriber_count(Coins::HYPE, key, ManualPriceDirection::Above),
        Some(1)
    );

    book.remove(Coins::HYPE, key, ManualPriceDirection::Above);

    assert!(!book.contains(Coins::HYPE, key, ManualPriceDirection::Above));
    assert_eq!(
        book.subscriber_count(Coins::HYPE, key, ManualPriceDirection::Above),
        None
    );
}

#[test]
fn delete_level_removes_entire_slot_regardless_of_subscriber_count() {
    let mut book = PriceBook::new();
    let key = price_key(TEST_TRIGGER_PRICE);

    book.insert(
        Coins::HYPE,
        key,
        ManualPriceDirection::Above,
        entry(TEST_TRIGGER_PRICE),
    );
    book.insert(
        Coins::HYPE,
        key,
        ManualPriceDirection::Above,
        entry(TEST_TRIGGER_PRICE),
    );

    assert_eq!(
        book.subscriber_count(Coins::HYPE, key, ManualPriceDirection::Above),
        Some(2)
    );

    let removed = book
        .delete_level(Coins::HYPE, key, ManualPriceDirection::Above)
        .unwrap();

    assert_eq!(removed.trigger_price, TEST_TRIGGER_PRICE);
    assert!(!book.contains(Coins::HYPE, key, ManualPriceDirection::Above));
    assert_eq!(
        book.subscriber_count(Coins::HYPE, key, ManualPriceDirection::Above),
        None
    );
}

#[test]
fn same_price_with_different_direction_is_different_level() {
    let mut book = PriceBook::new();
    let key = price_key(TEST_TRIGGER_PRICE);

    book.insert(
        Coins::HYPE,
        key,
        ManualPriceDirection::Above,
        entry(TEST_TRIGGER_PRICE),
    );
    book.insert(
        Coins::HYPE,
        key,
        ManualPriceDirection::Below,
        entry(TEST_TRIGGER_PRICE),
    );

    assert_eq!(
        book.subscriber_count(Coins::HYPE, key, ManualPriceDirection::Above),
        Some(1)
    );
    assert_eq!(
        book.subscriber_count(Coins::HYPE, key, ManualPriceDirection::Below),
        Some(1)
    );
}

#[test]
fn crossed_above_returns_levels_in_range() {
    let mut book = PriceBook::new();

    book.insert(
        Coins::HYPE,
        price_key(41.0),
        ManualPriceDirection::Above,
        entry(41.0),
    );
    book.insert(
        Coins::HYPE,
        price_key(43.0),
        ManualPriceDirection::Above,
        entry(43.0),
    );

    let levels = book.levels_crossed_above(Coins::HYPE, 40.0, 42.0);

    assert_eq!(levels.len(), 1);
    assert_eq!(levels[0].1.trigger_price, 41.0);
}

#[test]
fn crossed_below_returns_levels_in_range() {
    let mut book = PriceBook::new();

    book.insert(
        Coins::HYPE,
        price_key(41.0),
        ManualPriceDirection::Below,
        entry(41.0),
    );
    book.insert(
        Coins::HYPE,
        price_key(39.0),
        ManualPriceDirection::Below,
        entry(39.0),
    );

    let levels = book.levels_crossed_below(Coins::HYPE, 42.0, 40.0);

    assert_eq!(levels.len(), 1);
    assert_eq!(levels[0].1.trigger_price, 41.0);
}

#[test]
fn no_crossing_returns_empty_levels() {
    let mut book = PriceBook::new();

    book.insert(
        Coins::HYPE,
        price_key(TEST_TRIGGER_PRICE),
        ManualPriceDirection::Above,
        entry(TEST_TRIGGER_PRICE),
    );
    book.insert(
        Coins::HYPE,
        price_key(TEST_TRIGGER_PRICE),
        ManualPriceDirection::Below,
        entry(TEST_TRIGGER_PRICE),
    );

    assert!(book
        .levels_crossed_above(Coins::HYPE, 43.0, 41.0)
        .is_empty());
    assert!(book
        .levels_crossed_below(Coins::HYPE, 41.0, 43.0)
        .is_empty());
}
