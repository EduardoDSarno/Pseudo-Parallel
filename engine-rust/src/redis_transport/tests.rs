use redis::AsyncCommands;
use tokio::{
    sync::mpsc,
    time::{timeout, Duration},
};

use crate::{
    market_data::{
        alert_subscriptions::command::{SubscriptionCommand, SubscriptionManager},
        signal::price::ManualPriceDirection,
        types::Coins,
    },
    redis_transport::{
        constants::REDIS_ADDRESS, convert::to_subscription_manager, incoming::IncomingSubscription,
        subscription_listener::RedisSubscriptionListener,
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

async fn assert_no_message(receiver: &mut mpsc::Receiver<SubscriptionManager>) {
    assert!(timeout(Duration::from_millis(25), receiver.recv())
        .await
        .is_err());
}

#[test]
fn deserializes_price_subscription() {
    let json = r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "price",
            "coin": "HYPE",
            "trigger_price": 69.3,
            "direction": "below"
        }
    }"#;

    let incoming: IncomingSubscription = serde_json::from_str(json).unwrap();
    let manager = to_subscription_manager(incoming).unwrap();

    assert_eq!(manager.command, SubscriptionCommand::Subscribe);
    assert_eq!(manager.price.coin, Coins::HYPE);
    assert_eq!(manager.price.trigger_price, 69.3);
    assert_eq!(manager.price.direction, Some(ManualPriceDirection::Below));
}

#[test]
fn deserializes_price_subscription_without_direction() {
    let json = r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "price",
            "coin": "HYPE",
            "trigger_price": 70.0
        }
    }"#;

    let incoming: IncomingSubscription = serde_json::from_str(json).unwrap();
    let manager = to_subscription_manager(incoming).unwrap();

    assert_eq!(manager.price.coin, Coins::HYPE);
    assert_eq!(manager.price.trigger_price, 70.0);
    assert_eq!(manager.price.direction, None);
}

#[tokio::test]
async fn run_subscription_sends_price_manager() {
    let (sender, mut receiver) = mpsc::channel(1);

    RedisSubscriptionListener::run_subscription(price_payload(), &sender).await;

    let manager = receiver.recv().await.unwrap();
    assert_eq!(manager.command, SubscriptionCommand::Subscribe);
    assert_eq!(manager.price.coin, Coins::HYPE);
    assert_eq!(manager.price.trigger_price, 70.0);
    assert_eq!(manager.price.direction, None);
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

    RedisSubscriptionListener::run_subscription(payload.to_string(), &sender).await;

    let manager = receiver.recv().await.unwrap();
    assert_eq!(manager.price.direction, Some(ManualPriceDirection::Above));
}

#[tokio::test]
async fn run_subscription_skips_invalid_json() {
    let (sender, mut receiver) = mpsc::channel(1);

    RedisSubscriptionListener::run_subscription("not json".to_string(), &sender).await;

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

    RedisSubscriptionListener::run_subscription(payload.to_string(), &sender).await;

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

    RedisSubscriptionListener::run_subscription(payload.to_string(), &sender).await;

    assert_no_message(&mut receiver).await;
}

#[tokio::test]
async fn run_subscription_does_not_panic_when_receiver_closed() {
    let (sender, receiver) = mpsc::channel(1);
    drop(receiver);

    RedisSubscriptionListener::run_subscription(price_payload(), &sender).await;
}

#[tokio::test]
#[ignore = "requires local Redis running on REDIS_ADDRESS"]
async fn live_redis_publish_reaches_mpsc_receiver() {
    let channel = format!("alert_subscriptions_test_{}", std::process::id());
    let (sender, mut receiver) = mpsc::channel(1);
    let listener = RedisSubscriptionListener::new(REDIS_ADDRESS, sender).unwrap();

    let listener = tokio::spawn({
        let channel = channel.clone();
        async move {
            let _ = listener.bind_to_stream(&channel).await;
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
