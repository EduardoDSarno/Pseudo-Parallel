use crate::market_data::{
    signal::indicator_rules::{AtrRule, IndicatorRuleKind, IndicatorRuleService},
    types::{CandleKey, Coins, Interval},
};

fn key(interval: Interval) -> CandleKey {
    CandleKey::new(Coins::HYPE, interval)
}

fn atr_rule() -> IndicatorRuleKind {
    IndicatorRuleKind::Atr(AtrRule {
        breakout_ratio: 2.5,
        debug_ratio: 0.8,
    })
}

#[test]
fn subscribing_rules_assigns_unique_ids() {
    let mut service = IndicatorRuleService::new();

    let first = service.subscribe(key(Interval::M5), atr_rule());
    let second = service.subscribe(key(Interval::M15), atr_rule());

    assert_ne!(first.id, second.id);
}

#[test]
fn rules_are_returned_only_for_matching_key() {
    let mut service = IndicatorRuleService::new();
    let m5 = key(Interval::M5);
    let h1 = key(Interval::H1);

    service.subscribe(m5.clone(), atr_rule());

    assert_eq!(service.rules_for_key(&m5).len(), 1);
    assert!(service.rules_for_key(&h1).is_empty());
}

#[test]
fn unsubscribe_removes_matching_rule_and_clears_empty_key() {
    let mut service = IndicatorRuleService::new();
    let m5 = key(Interval::M5);
    let kind = atr_rule();

    let rule = service.subscribe(m5.clone(), kind.clone());
    assert_eq!(service.rules_for_key(&m5).len(), 1);

    let removed = service.unsubscribe(m5.clone(), kind).unwrap();
    assert_eq!(removed.id, rule.id);
    assert!(service.rules_for_key(&m5).is_empty());
}

#[test]
fn unsubscribe_errors_when_rule_missing() {
    let mut service = IndicatorRuleService::new();
    let err = service
        .unsubscribe(key(Interval::M5), atr_rule())
        .unwrap_err();
    assert_eq!(err.to_string(), "indicator rule not found");
}

#[test]
fn subscribe_default_atr_rules_creates_one_rule_per_key() {
    let mut service = IndicatorRuleService::new();
    let keys = [key(Interval::M5), key(Interval::M15), key(Interval::H1)];

    let ids = service.subscribe_default_atr_rules(&keys, 2.5, 0.8);

    assert_eq!(ids.len(), keys.len());
    for key in keys {
        assert_eq!(service.rules_for_key(&key).len(), 1);
    }
}
