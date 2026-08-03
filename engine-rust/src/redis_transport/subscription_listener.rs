use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::{
    market_data::alert_subscriptions::command::SubscriptionManager,
    redis_transport::{
        constants::{
            REDIS_RECONNECT_BACKOFF_MULTIPLIER, REDIS_RECONNECT_INITIAL_MS, REDIS_RECONNECT_MAX_MS,
        },
        convert::to_subscription_manager,
        incoming::IncomingSubscription,
    },
};

/* Redis side of the subscription pipe — TS backend PUBLISHes JSON here,
we SUBSCRIBE, parse it, and forward to the mpsc channel that run_live reads. */
pub struct RedisSubscriptionListener {
    redis_client: redis::Client, // connection to redis server (localhost in dev)
    subscription_sender: mpsc::Sender<SubscriptionManager>, // sender end — live_loop owns the receiver
}

impl RedisSubscriptionListener {
    /* Build the stream listener — main passes the mpsc sender from the channel it created with run_live */
    pub fn new(
        address: impl Into<String>,
        subscription_sender: mpsc::Sender<SubscriptionManager>,
    ) -> Result<Self, redis::RedisError> {
        Ok(Self {
            redis_client: redis::Client::open(address.into())?,
            subscription_sender,
        })
    }

    /* Subscribe to a redis channel and loop forever on incoming messages.
    Each message is JSON from backend-ts (same shape as incoming.rs).
    Parsed subs go on mpsc → run_live select! recv arm → apply_subscription. */
    pub async fn bind_to_stream(self, channel: &str) -> Result<(), Box<dyn std::error::Error>> {
        // pub/sub needs its own connection — not the same as normal redis commands
        let mut pubsub = self.redis_client.get_async_pubsub().await?;
        pubsub.subscribe(channel).await?;

        // on_message returns an async stream of Msg (one per PUBLISH from TS)
        let mut stream = pubsub.on_message();
        // clone the sender handle for the loop — Sender is cheap to copy
        let subscription_sender = self.subscription_sender;

        while let Some(msg) = stream.next().await
        // wait for next message
        {
            let payload: String = msg.get_payload()?; // Msg → string (our JSON)
            tracing::info!(payload = %payload, "redis subscription message received");
            Self::run_subscription(payload, &subscription_sender).await;
        }

        Ok(())
    }

    /* Parse one redis payload and push a SubscriptionManager on the mpsc channel.
    Bad JSON or convert errors are logged and skipped — we don't crash the listener. */
    pub(crate) async fn run_subscription(
        payload: String,
        subscription_sender: &mpsc::Sender<SubscriptionManager>,
    ) {
        // serde JSON string → IncomingSubscription (wire type from incoming.rs)
        let incoming = match IncomingSubscription::parse_message(&payload) {
            Ok(sub) => sub,
            Err(err) => {
                tracing::warn!(error = %err, payload = %payload, "invalid JSON, skipping");
                return;
            }
        };

        // IncomingSubscription → SubscriptionManager (convert.rs — same as we would call from API)
        let manager = match to_subscription_manager(incoming) {
            Ok(manager) => manager,
            Err(err) => {
                tracing::warn!(error = %err, "convert failed, skipping");
                return;
            }
        };

        // send to run_live — it will process(MarketUpdate::Subscription(manager))
        if let Err(err) = subscription_sender.send(manager).await {
            tracing::warn!(error = %err, "mpsc send failed");
        }
    }
}

/* Keeps subscription delivery alive across redis drops — bind_to_stream only ever
returns when the connection/stream ends, so each time it does, this rebuilds a fresh
listener (new client, new pubsub connection) and retries with exponential backoff
instead of letting the caller's task end permanently. */
pub async fn run_with_reconnect(
    address: String,
    channel: String,
    subscription_sender: mpsc::Sender<SubscriptionManager>,
) {
    let mut backoff = Duration::from_millis(REDIS_RECONNECT_INITIAL_MS);

    loop {
        let listener =
            match RedisSubscriptionListener::new(address.clone(), subscription_sender.clone()) {
                Ok(listener) => listener, // happy path — we have a listener
                Err(err) => {
                    tracing::error!(error = %err, "failed to open redis subscription connection, retrying");
                    tokio::time::sleep(backoff).await;
                    backoff = next_backoff(backoff);
                    continue;
                }
            };

        // calls bind_to_stream, which only returns when the connection/stream ends
        match listener.bind_to_stream(&channel).await {
            Ok(()) => tracing::warn!("redis subscription stream ended, reconnecting"),
            Err(err) => {
                tracing::error!(error = %err, "redis subscription listener stopped, reconnecting")
            }
        }

        tokio::time::sleep(backoff).await;
        backoff = next_backoff(backoff);
    }
}

fn next_backoff(current: Duration) -> Duration {
    let doubled = current.saturating_mul(REDIS_RECONNECT_BACKOFF_MULTIPLIER);
    let max = Duration::from_millis(REDIS_RECONNECT_MAX_MS);
    doubled.min(max)
}
