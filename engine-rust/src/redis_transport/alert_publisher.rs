use redis::AsyncCommands;
use tokio::sync::mpsc;

/* Redis side of the outgoing alert pipe — dispatch_alerts pushes JSON strings
onto an unbounded channel; this task publishes each one for the TS backend. */
pub struct RedisAlertPublisher {
    connection: redis::aio::MultiplexedConnection,
    channel: String,
}

impl RedisAlertPublisher {
    pub async fn new(
        address: impl Into<String>,
        channel: impl Into<String>,
    ) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(address.into())?;
        let connection = client.get_multiplexed_async_connection().await?;
        Ok(Self {
            connection,
            channel: channel.into(),
        })
    }

    /* Drains the receiver forever, publishing each message. Bad publishes are logged
    and skipped — one failed alert shouldn't stop the rest from going out. */
    async fn run(mut self, mut receiver: mpsc::UnboundedReceiver<String>) {
        while let Some(message) = receiver.recv().await {
            if let Err(err) = self
                .connection
                .publish::<_, _, ()>(self.channel.as_str(), message)
                .await
            {
                tracing::error!(error = %err, "failed to publish alert to redis");
            }
        }

        tracing::warn!("alert publisher channel closed, stopping");
    }
}

/* Builds the publisher, spawns its background task, and returns the sender used by dispatch. */
pub async fn spawn_alert_publisher(
    address: impl Into<String>,
    channel: impl Into<String>,
) -> Result<mpsc::UnboundedSender<String>, redis::RedisError> {
    let publisher = RedisAlertPublisher::new(address, channel).await?;
    let (sender, receiver) = mpsc::unbounded_channel();

    tokio::spawn(publisher.run(receiver));

    Ok(sender)
}
