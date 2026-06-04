use crate::market_data::{
    constans::{DEFAULT_ATR_BREAKOUT_RATIO, DEFAULT_LIVE_ATR_DEBUG_RATIO},
    signal::{
        indicator_rules::{
            indicator::Indicator, AtrRule, IndicatorRuleKind,
        },
        price::{alert::ManualPriceAlert, ManualPriceDirection},
    },
    alert_subscriptions::command::{SubscriptionCommand, SubscriptionManager, SubscriptionType},
    types::{CandleKey, Coins, Interval},
};

const TEST_BELOW_697: f64 = 69.3;
const TEST_BELOW_700: f64 = 69.7;

/* Dev-only signal subscriptions until Redis/API loader exists. */
pub fn dev_signal_subscriptions() -> Vec<SubscriptionManager> {
    let coin = Coins::HYPE;
    let default_atr = IndicatorRuleKind::Atr(AtrRule {
        breakout_ratio: DEFAULT_ATR_BREAKOUT_RATIO,
        debug_ratio: DEFAULT_LIVE_ATR_DEBUG_RATIO,
    });

    vec![
        subscribe_price(coin, TEST_BELOW_697, ManualPriceDirection::Below),
        subscribe_price(coin, TEST_BELOW_700, ManualPriceDirection::Below),
        subscribe_indicator(CandleKey::new(coin, Interval::M5), default_atr.clone()),
        subscribe_indicator(CandleKey::new(coin, Interval::M15), default_atr.clone()),
        subscribe_indicator(CandleKey::new(coin, Interval::H1), default_atr),
    ]
}

fn subscribe_price(
    coin: Coins,
    trigger_price: f64,
    direction: ManualPriceDirection,
) -> SubscriptionManager {
    SubscriptionManager {
        command: SubscriptionCommand::Subscribe,
        sub_type: SubscriptionType::Price(ManualPriceAlert::new(coin, trigger_price, direction)),
    }
}

fn subscribe_indicator(key: CandleKey, kind: IndicatorRuleKind) -> SubscriptionManager {
    SubscriptionManager {
        command: SubscriptionCommand::Subscribe,
        sub_type: SubscriptionType::Indicator(Indicator::new(key, kind)),
    }
}
