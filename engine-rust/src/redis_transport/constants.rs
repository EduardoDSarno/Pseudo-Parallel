pub const REDIS_ADDRESS: &str = "redis://127.0.0.1:6379";
pub const SUBSCRIPTION_CHANNEL: &str = "alert_subscriptions"; // match TS
pub const ALERTS_FIRED_CHANNEL: &str = "alerts_fired"; // match TS

pub const REDIS_RECONNECT_INITIAL_MS: u64 = 1_000;
pub const REDIS_RECONNECT_MAX_MS: u64 = 30_000;
pub const REDIS_RECONNECT_BACKOFF_MULTIPLIER: u32 = 2;
