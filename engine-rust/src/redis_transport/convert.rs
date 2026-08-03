/* Turns incoming Redis messages into SubscriptionManager so market data can apply them. */

use std::error::Error;

use crate::{
    market_data::{
        alert_subscriptions::command::{
            PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager,
        },
        signal::price::ManualPriceDirection,
    },
    redis_transport::incoming::{
        IncomingPriceSubscription, IncomingSubscription, IncomingSubscriptionType,
    },
};

/* Entry point after serde — send result on the channel or call apply_subscription. */
pub fn to_subscription_manager(
    incoming: IncomingSubscription,
) -> Result<SubscriptionManager, Box<dyn Error>> {
    let IncomingSubscriptionType::Price(price) = incoming.sub_type;
    Ok(SubscriptionManager {
        command: parse_command(&incoming.command)?,
        price: to_price_subscription_spec(price)?,
    })
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
