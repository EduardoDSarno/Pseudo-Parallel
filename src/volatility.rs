use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use chrono_tz::America::New_York;
use hypersdk::{Decimal, dec};

/// This are percentage points related to BTC's price
const NEW_YORK_OPEN_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.45);
const WEEKEND_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.20);
const NORMAL_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.30);

const MINUTES_PER_HOUR: u32 = 60;
const NEW_YORK_MARKET_OPEN_MINUTES: u32 = 9 * MINUTES_PER_HOUR + 30;
const NEW_YORK_OPEN_WINDOW_END_MINUTES: u32 = 11 * MINUTES_PER_HOUR;
const VOLATILITY_COOLDOWN_SECONDS: u64 = 60;

/// This enum will hold the value threshol of the different volatility types
/// that I chose to cover
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketTimesVol {
    /// Covers time that New York exchange is open
    NewYorkOpen,
    /// Covers weekends
    Weekend,
    /// Covers the rest the first 2 don't
    Normal,
}

impl MarketTimesVol {
    fn threshold_percent(self) -> Decimal {
        match self {
            Self::NewYorkOpen => NEW_YORK_OPEN_VOLATILITY_THRESHOLD_PERCENT,
            Self::Weekend => WEEKEND_VOLATILITY_THRESHOLD_PERCENT,
            Self::Normal => NORMAL_VOLATILITY_THRESHOLD_PERCENT,
        }
    }
}

use std::time::{Duration, Instant, SystemTime};

use crate::market::Coin;

/// Describes a price movement that crossed the active volatility threshold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolatilitySpike {
    pub coin: Coin,
    pub percent_change: Decimal,
    pub threshold_percent: Decimal,
    pub market_time: MarketTimesVol,
    pub observed_at: SystemTime,
}

impl VolatilitySpike {
    pub fn display(&self) {
        log::warn!(
            "{} price spike: {}% (threshold: {}%, market time: {:?})",
            self.coin,
            self.percent_change,
            self.threshold_percent,
            self.market_time
        );
    }
}

/// This struct is just to hold the time when a cooldown_ends
pub struct VolatilityDetector {
    cooldown_until: Option<Instant>,
}

impl VolatilityDetector {
    pub fn new() -> Self {
        Self {
            cooldown_until: None,
        }
    }

    /// This returns true if the value is Some and cooldown is still running
    fn cooldown_is_active(&self) -> bool {
        self.cooldown_until
            .is_some_and(|until| Instant::now() < until)
    }
}

/// Matches the current time to a market type
pub fn match_market_time(timestamp: SystemTime) -> MarketTimesVol {
    let utc_time: DateTime<Utc> = timestamp.into();
    let new_york_time = utc_time.with_timezone(&New_York);

    if matches!(new_york_time.weekday(), Weekday::Sat | Weekday::Sun) {
        return MarketTimesVol::Weekend;
    }

    let minutes_after_midnight = new_york_time.hour() * MINUTES_PER_HOUR + new_york_time.minute();

    if (NEW_YORK_MARKET_OPEN_MINUTES..NEW_YORK_OPEN_WINDOW_END_MINUTES)
        .contains(&minutes_after_midnight)
    {
        MarketTimesVol::NewYorkOpen
    } else {
        MarketTimesVol::Normal
    }
}

/// Evaluates a price movement and returns its spike information when the
/// movement crosses the threshold for the observed market time.
pub fn evaluate_volatility(
    coin: Coin,
    percent_change: Decimal,
    observed_at: SystemTime,
    detector: &mut VolatilityDetector,
) -> Option<VolatilitySpike> {
    // ignore alert
    if detector.cooldown_is_active() {
        return None;
    }

    let current_mkt_time = match_market_time(observed_at);
    let threshold_prct = current_mkt_time.threshold_percent();

    // using absolute for dowards and upwards spikes
    if percent_change.abs() >= threshold_prct {
        detector.cooldown_until =
            Some(Instant::now() + Duration::from_secs(VOLATILITY_COOLDOWN_SECONDS));

        return Some(VolatilitySpike {
            coin,
            percent_change,
            threshold_percent: threshold_prct,
            market_time: current_mkt_time,
            observed_at,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use hypersdk::dec;

    use crate::market::Coin;

    use super::{MarketTimesVol, VolatilityDetector, evaluate_volatility, match_market_time};

    fn timestamp(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    #[test]
    fn matches_new_york_open_during_summer() {
        assert_eq!(
            match_market_time(timestamp(1_786_975_200)), // Monday 10:00 EDT
            MarketTimesVol::NewYorkOpen
        );
    }

    #[test]
    fn matches_new_york_open_during_winter() {
        assert_eq!(
            match_market_time(timestamp(1_768_230_000)), // Monday 10:00 EST
            MarketTimesVol::NewYorkOpen
        );
    }

    #[test]
    fn matches_sunday_as_weekend() {
        assert_eq!(
            match_market_time(timestamp(1_786_896_000)), // Sunday 12:00 EDT
            MarketTimesVol::Weekend
        );
    }

    #[test]
    fn matches_saturday_as_weekend() {
        assert_eq!(
            match_market_time(timestamp(1_768_064_400)), // Saturday 12:00 EST
            MarketTimesVol::Weekend
        );
    }

    #[test]
    fn matches_weekday_afternoon_as_normal() {
        assert_eq!(
            match_market_time(timestamp(1_786_993_200)), // Monday 15:00 EDT
            MarketTimesVol::Normal
        );
    }

    #[test]
    fn matches_weekday_before_open_as_normal() {
        assert_eq!(
            match_market_time(timestamp(1_768_226_400)), // Monday 09:00 EST
            MarketTimesVol::Normal
        );
    }

    #[test]
    fn active_cooldown_suppresses_spike() {
        let cooldown_until = Instant::now() + Duration::from_secs(60);
        let mut detector = VolatilityDetector {
            cooldown_until: Some(cooldown_until),
        };

        let spike = evaluate_volatility(
            Coin::Btc,
            dec!(5.00),
            timestamp(1_786_993_200),
            &mut detector,
        );

        assert!(spike.is_none());
        assert_eq!(detector.cooldown_until, Some(cooldown_until));
    }

    #[test]
    fn movement_below_normal_threshold_is_not_a_spike() {
        let mut detector = VolatilityDetector::new();

        let spike = evaluate_volatility(
            Coin::Btc,
            dec!(0.20),
            timestamp(1_786_993_200),
            &mut detector,
        );

        assert!(spike.is_none());
        assert!(detector.cooldown_until.is_none());
    }

    #[test]
    fn positive_movement_above_every_threshold_is_a_spike() {
        let mut detector = VolatilityDetector::new();

        let observed_at = timestamp(1_786_993_200);
        let spike = evaluate_volatility(Coin::Btc, dec!(1.00), observed_at, &mut detector)
            .expect("movement should produce a volatility spike");

        assert_eq!(spike.coin, Coin::Btc);
        assert_eq!(spike.percent_change, dec!(1.00));
        assert_eq!(spike.threshold_percent, dec!(0.30));
        assert_eq!(spike.market_time, MarketTimesVol::Normal);
        assert_eq!(spike.observed_at, observed_at);
        assert!(detector.cooldown_is_active());
    }

    #[test]
    fn negative_movement_above_every_threshold_is_a_spike() {
        let mut detector = VolatilityDetector::new();

        let observed_at = timestamp(1_786_993_200);
        let spike = evaluate_volatility(Coin::Btc, dec!(-1.00), observed_at, &mut detector)
            .expect("movement should produce a volatility spike");

        assert_eq!(spike.percent_change, dec!(-1.00));
        assert_eq!(spike.observed_at, observed_at);
        assert!(detector.cooldown_is_active());
    }
}
