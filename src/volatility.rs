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

/// Received a percent change and a mutable detector and evaluates if a price spiked
/// and updates the detector based on that
/// Later add a struct insated of a bool for returning type, preferably something like Option<VolatilitySpike>
pub fn evaluate_volatility(percent_change: Decimal, detector: &mut VolatilityDetector) -> bool {
    // ignore alert
    if detector.cooldown_is_active() {
        return false;
    }

    let current_mkt_time = match_market_time(SystemTime::now());
    let threshold_prct = current_mkt_time.threshold_percent();

    // using absolute for dowards and upwards spikes
    if percent_change.abs() >= threshold_prct {
        log::warn!("PRICE SPIKE DETECTED");
        detector.cooldown_until =
            Some(Instant::now() + Duration::from_secs(VOLATILITY_COOLDOWN_SECONDS));
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use hypersdk::dec;

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

        let detected = evaluate_volatility(dec!(5.00), &mut detector);

        assert!(!detected);
        assert_eq!(detector.cooldown_until, Some(cooldown_until));
    }

    #[test]
    fn movement_below_every_threshold_is_not_a_spike() {
        let mut detector = VolatilityDetector::new();

        let detected = evaluate_volatility(dec!(0.20), &mut detector);

        assert!(!detected);
        assert!(detector.cooldown_until.is_none());
    }

    #[test]
    fn positive_movement_above_every_threshold_is_a_spike() {
        let mut detector = VolatilityDetector::new();

        let detected = evaluate_volatility(dec!(1.00), &mut detector);

        assert!(detected);
        assert!(detector.cooldown_is_active());
    }

    #[test]
    fn negative_movement_above_every_threshold_is_a_spike() {
        let mut detector = VolatilityDetector::new();

        let detected = evaluate_volatility(dec!(-1.00), &mut detector);

        assert!(detected);
        assert!(detector.cooldown_is_active());
    }
}
