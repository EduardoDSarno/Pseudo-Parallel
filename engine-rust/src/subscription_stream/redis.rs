use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::market_data::alert_subscriptions::{
    command::SubscriptionManager, convert::to_subscription_manager, incoming::IncomingSubscription,
};

/* Redis side of the subscription pipe — TS backend PUBLISHes JSON here,
we SUBSCRIBE, parse it, and forward to the mpsc channel that run_live reads. */
pub struct RedisSubscriptionStream {
    redis_client: redis::Client, // connection to redis server (localhost in dev)
    subscription_sender: mpsc::Sender<SubscriptionManager>, // sender end — live_loop owns the receiver
}

impl RedisSubscriptionStream {
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

