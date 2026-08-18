use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hypersdk::{Decimal, dec};

// Channel capacities.
pub const MARKET_INPUT_BUFFER: usize = 256;
pub const ACCOUNT_LOOKUP_BUFFER: usize = 2_048;
pub const POSITION_UPDATE_BUFFER: usize = 256;

// Hyperliquid REST pacing. A 125 ms gap allows at most eight requests/second.
pub const CLEARINGHOUSE_REQUEST_INTERVAL: Duration = Duration::from_millis(125);
pub const MINIMUM_WHALE_POSITION_USD: Decimal = dec!(1_000_000);

// Rolling volatility window.
pub const VOLATILITY_WINDOW_DURATION: Duration = Duration::from_secs(60);
pub const VOLATILITY_WINDOW_MAX_POINTS: usize = 1_000;
pub const VOLATILITY_COOLDOWN: Duration = Duration::from_secs(60);

// Volatility thresholds expressed in percentage points.
pub const NEW_YORK_OPEN_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.45);
pub const WEEKEND_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.20);
pub const NORMAL_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.30);

/// Converts a millisecond timestamp supplied by Hyperliquid into Rust's
/// `SystemTime`. Returns `None` if the timestamp exceeds the supported range.
pub fn hyperliquid_time_to_system_time(timestamp_ms: u64) -> Option<SystemTime> {
    UNIX_EPOCH.checked_add(Duration::from_millis(timestamp_ms))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::hyperliquid_time_to_system_time;

    #[test]
    fn converts_hyperliquid_milliseconds_to_system_time() {
        let timestamp = hyperliquid_time_to_system_time(1_754_345_600_123);

        assert_eq!(
            timestamp,
            Some(UNIX_EPOCH + Duration::from_millis(1_754_345_600_123))
        );
    }
}
