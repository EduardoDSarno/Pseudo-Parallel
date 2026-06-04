/* Turns incoming subs into SubscriptionManager so apply / mpsc can use them. */

use std::error::Error;

use crate::market_data::{
    alert_subscriptions::{
        command::{
            PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager, SubscriptionType,
        },
        incoming::{
            IncomingAtrRule, IncomingIndicatorKind, IncomingIndicatorSubscription,
            IncomingPriceSubscription, IncomingSubscription, IncomingSubscriptionType,
        },
    },
    signal::{
        indicator_rules::{
            indicator::Indicator, AtrRule, IndicatorRuleKind,
        },
        price::ManualPriceDirection,
    },
    types::CandleKey,
};

/* Entry point after serde — send result on the channel or call apply_subscription. */
pub fn to_subscription_manager(
    incoming: IncomingSubscription,
) -> Result<SubscriptionManager, Box<dyn Error>> {
    Ok(SubscriptionManager {
        command: parse_command(&incoming.command)?,
        sub_type: to_subscription_type(incoming.sub_type)?,
    })
}

/* Price vs indicator branch. */
fn to_subscription_type(
    sub_type: IncomingSubscriptionType,
) -> Result<SubscriptionType, Box<dyn Error>> {
    match sub_type {
        IncomingSubscriptionType::Price(p) => {
            Ok(SubscriptionType::Price(to_price_subscription_spec(p)?))
        }
        IncomingSubscriptionType::Indicator(i) => {
            Ok(SubscriptionType::Indicator(to_indicator(i)?))
        }
    }
}

/* Wire price fields -> spec; omit direction to infer in apply_subscription. */
fn to_price_subscription_spec(
    incoming: IncomingPriceSubscription,
) -> Result<PriceSubscriptionSpec, Box<dyn Error>> {
    let direction = match incoming.direction {
        Some(s) if !s.trim().is_empty() => Some(parse_direction(&s)?),
        _ => None,
    };
    Ok(PriceSubscriptionSpec {
        coin: incoming.coin,
        trigger_price: incoming.trigger_price,
        direction,
    })
}

/* Wire indicator fields -> Indicator (candle key + rule kind). */
fn to_indicator(incoming: IncomingIndicatorSubscription) -> Result<Indicator, Box<dyn Error>> {
    let key = CandleKey::new(incoming.coin, incoming.interval);
    Ok(Indicator::new(
        key,
        to_indicator_rule_kind(incoming.kind),
    ))
}

/* Match incoming indicator type to engine IndicatorRuleKind — add arms when we add indicators. */
fn to_indicator_rule_kind(kind: IncomingIndicatorKind) -> IndicatorRuleKind {
    match kind {
        IncomingIndicatorKind::Atr(atr) => IndicatorRuleKind::Atr(to_atr_rule(atr)),
    }
}

/* ATR params from incoming into AtrRule. */
fn to_atr_rule(incoming: IncomingAtrRule) -> AtrRule {
    AtrRule {
        breakout_ratio: incoming.breakout_ratio,
        debug_ratio: incoming.debug_ratio,
    }
}

/* External command string -> Subscribe / Unsubscribe. */
fn parse_command(command: &str) -> Result<SubscriptionCommand, Box<dyn Error>> {
    match command.to_lowercase().as_str() {
        "subscribe" => Ok(SubscriptionCommand::Subscribe),
        "unsubscribe" => Ok(SubscriptionCommand::Unsubscribe),
        other => Err(format!("unknown subscription command: {other}").into()),
    }
}

/* External direction string -> Above / Below. */
fn parse_direction(direction: &str) -> Result<ManualPriceDirection, Box<dyn Error>> {
    match direction.to_lowercase().as_str() {
        "above" => Ok(ManualPriceDirection::Above),
        "below" => Ok(ManualPriceDirection::Below),
        other => Err(format!("unknown price direction: {other}").into()),
    }
}
