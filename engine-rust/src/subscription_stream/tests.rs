use redis::AsyncCommands;
use tokio::{
    sync::mpsc,
    time::{timeout, Duration},
};

use crate::{
    market_data::{
        alert_subscriptions::command::{SubscriptionCommand, SubscriptionManager, SubscriptionType},
        signal::{
            indicator_rules::{AtrRule, IndicatorRuleKind},
            price::ManualPriceDirection,
        },
        types::{Coins, Interval},
    },
    subscription_stream::{
        redis::RedisSubscriptionStream,
        subscription_constants::REDIS_ADDRESS,
    },
};

fn price_payload() -> String {
    r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "price",
            "coin": "HYPE",
            "trigger_price": 70.0
        }
    }"#
    .to_string()
}

fn indicator_payload() -> String {
    r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "indicator",
            "coin": "HYPE",
            "interval": "5m",
            "kind": {
                "type": "atr",
                "breakout_ratio": 2.5,
                "debug_ratio": 0.8
            }
        }
    }"#
    .to_string()
}

async fn assert_no_message(receiver: &mut mpsc::Receiver<SubscriptionManager>) {
    assert!(timeout(Duration::from_millis(25), receiver.recv())
        .await
        .is_err());
}

#[tokio::test]
async fn run_subscription_sends_price_manager() {
    let (sender, mut receiver) = mpsc::channel(1);

    RedisSubscriptionStream::run_subscription(price_payload(), &sender).await;

    let manager = receiver.recv().await.unwrap();
    assert_eq!(manager.command, SubscriptionCommand::Subscribe);
    match manager.sub_type {
        SubscriptionType::Price(spec) => {
            assert_eq!(spec.coin, Coins::HYPE);
            assert_eq!(spec.trigger_price, 70.0);
            assert_eq!(spec.direction, None);
        }
        SubscriptionType::Indicator(_) => panic!("expected price subscription"),
    }
}

#[tokio::test]
async fn run_subscription_sends_indicator_manager() {
    let (sender, mut receiver) = mpsc::channel(1);

    RedisSubscriptionStream::run_subscription(indicator_payload(), &sender).await;

    let manager = receiver.recv().await.unwrap();
    assert_eq!(manager.command, SubscriptionCommand::Subscribe);
    match manager.sub_type {
        SubscriptionType::Indicator(ind) => {
            assert_eq!(ind.key.coin, Coins::HYPE);
            assert_eq!(ind.key.interval, Interval::M5);
            assert_eq!(
                ind.kind,
                IndicatorRuleKind::Atr(AtrRule {
                    breakout_ratio: 2.5,
                    debug_ratio: 0.8,
                })
            );
        }
        SubscriptionType::Price(_) => panic!("expected indicator subscription"),
    }
}

#[tokio::test]
async fn run_subscription_sends_price_manager_with_explicit_direction() {
    let (sender, mut receiver) = mpsc::channel(1);
    let payload = r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "price",
            "coin": "HYPE",
            "trigger_price": 70.0,
            "direction": "above"
        }
    }"#;

    RedisSubscriptionStream::run_subscription(payload.to_string(), &sender).await;

    let manager = receiver.recv().await.unwrap();
    match manager.sub_type {
        SubscriptionType::Price(spec) => {
            assert_eq!(spec.direction, Some(ManualPriceDirection::Above));
        }
        SubscriptionType::Indicator(_) => panic!("expected price subscription"),
    }
}

#[tokio::test]
async fn run_subscription_skips_invalid_json() {
    let (sender, mut receiver) = mpsc::channel(1);

    RedisSubscriptionStream::run_subscription("not json".to_string(), &sender).await;

    assert_no_message(&mut receiver).await;
}

#[tokio::test]
async fn run_subscription_skips_unknown_command() {
    let (sender, mut receiver) = mpsc::channel(1);
    let payload = r#"{
        "command": "pause",
        "sub_type": {
            "type": "price",
            "coin": "HYPE",
            "trigger_price": 70.0
        }
    }"#;

    RedisSubscriptionStream::run_subscription(payload.to_string(), &sender).await;

    assert_no_message(&mut receiver).await;
}

#[tokio::test]
async fn run_subscription_skips_bad_direction() {
    let (sender, mut receiver) = mpsc::channel(1);
    let payload = r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "price",
            "coin": "HYPE",
            "trigger_price": 70.0,
            "direction": "sideways"
        }
    }"#;

    RedisSubscriptionStream::run_subscription(payload.to_string(), &sender).await;

    assert_no_message(&mut receiver).await;
}

#[tokio::test]
async fn run_subscription_does_not_panic_when_receiver_closed() {
    let (sender, receiver) = mpsc::channel(1);
    drop(receiver);

    RedisSubscriptionStream::run_subscription(price_payload(), &sender).await;
}

#[tokio::test]
#[ignore = "requires local Redis running on REDIS_ADDRESS"]
async fn live_redis_publish_reaches_mpsc_receiver() {
    let channel = format!("alert_subscriptions_test_{}", std::process::id());
    let (sender, mut receiver) = mpsc::channel(1);
    let stream = RedisSubscriptionStream::new(REDIS_ADDRESS, sender).unwrap();

    let listener = tokio::spawn({
        let channel = channel.clone();
        async move {
            let _ = stream.bind_to_stream(&channel).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = redis::Client::open(REDIS_ADDRESS).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let _: i64 = conn.publish(&channel, price_payload()).await.unwrap();

    let manager = timeout(Duration::from_secs(2), receiver.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(manager.command, SubscriptionCommand::Subscribe);
    listener.abort();
}
