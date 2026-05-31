use crate::market_data::{
    signal::evaluate::evaluators::price_evaluator::PriceEvaluator, types::Coins,
};

#[test]
fn evaluate_price_with_empty_service_returns_no_alerts() {
    let evaluator = PriceEvaluator::new();
    let service = crate::market_data::signal::price::PriceAlertService::new();

    assert!(evaluator
        .evaluate_price(&service, Coins::HYPE, 29.0, 30.0)
        .is_empty());
}
