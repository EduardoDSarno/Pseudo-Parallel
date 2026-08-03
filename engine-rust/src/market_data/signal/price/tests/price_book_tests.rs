use crate::market_data::{
    signal::price::{
        key::{ManualPriceDirection, PriceKey},
        price_book::{entry::PriceLevelEntry, PriceBook},
    },
    types::Coins,
};

fn key(price: f64) -> PriceKey {
    PriceKey::from_price(price).unwrap()
}

fn insert(book: &mut PriceBook, price: f64, direction: ManualPriceDirection) {
    book.insert(
        Coins::HYPE,
        key(price),
        direction,
        PriceLevelEntry::new(price),
    );
}

#[test]
fn shared_level_requires_two_removals() {
    let mut book = PriceBook::new();
    insert(&mut book, 42.0, ManualPriceDirection::Above);
    insert(&mut book, 42.0, ManualPriceDirection::Above);

    assert!(book
        .remove(Coins::HYPE, key(42.0), ManualPriceDirection::Above)
        .is_some());
    assert!(book
        .remove(Coins::HYPE, key(42.0), ManualPriceDirection::Above)
        .is_some());
    assert!(book
        .remove(Coins::HYPE, key(42.0), ManualPriceDirection::Above)
        .is_none());
}

#[test]
fn delete_level_removes_all_subscribers() {
    let mut book = PriceBook::new();
    insert(&mut book, 42.0, ManualPriceDirection::Above);
    insert(&mut book, 42.0, ManualPriceDirection::Above);

    assert!(book
        .delete_level(Coins::HYPE, key(42.0), ManualPriceDirection::Above)
        .is_some());
    assert!(book
        .delete_level(Coins::HYPE, key(42.0), ManualPriceDirection::Above)
        .is_none());
}

#[test]
fn crossed_above_returns_only_levels_in_range() {
    let mut book = PriceBook::new();
    insert(&mut book, 41.0, ManualPriceDirection::Above);
    insert(&mut book, 43.0, ManualPriceDirection::Above);

    let levels = book.levels_crossed_above(Coins::HYPE, 40.0, 42.0);

    assert_eq!(levels.len(), 1);
    assert_eq!(levels[0].1.trigger_price, 41.0);
}

#[test]
fn crossed_below_returns_only_levels_in_range() {
    let mut book = PriceBook::new();
    insert(&mut book, 41.0, ManualPriceDirection::Below);
    insert(&mut book, 39.0, ManualPriceDirection::Below);

    let levels = book.levels_crossed_below(Coins::HYPE, 42.0, 40.0);

    assert_eq!(levels.len(), 1);
    assert_eq!(levels[0].1.trigger_price, 41.0);
}
