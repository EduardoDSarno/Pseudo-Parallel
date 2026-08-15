use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use chrono_tz::{America::New_York};
use hypersdk::{Decimal, dec};

const NEW_YORK_OPEN_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.60);
const WEEKEND_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.30);
const NORMAL_VOLATILITY_THRESHOLD_PERCENT: Decimal = dec!(0.45);

const MINUTES_PER_HOUR: u32 = 60;
const NEW_YORK_MARKET_OPEN_MINUTES: u32 = 9 * MINUTES_PER_HOUR + 30;
const NEW_YORK_OPEN_WINDOW_END_MINUTES: u32 = 11 * MINUTES_PER_HOUR;
const VOLATILITY_COOLDOWN_SECONDS: u64 = 60;

/// This enum will hold the value threshol of the different volatility types 
/// that I chose to cover
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketTimesVol 
{
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
pub fn match_market_time(timestamp: SystemTime) -> MarketTimesVol 
{

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
pub fn evaluate_volatility(percent_change: Decimal, detector: &mut VolatilityDetector)  -> bool
{
    // ignore alert
    if detector.cooldown_is_active() 
    {
        return false;
    }

    let current_mkt_time = match_market_time(SystemTime::now());
    let threshold_prct = current_mkt_time.threshold_percent();

    // using absolute for dowards and upwards spikes
    if percent_change.abs() >= threshold_prct
    {
        log::warn!("PRICE SPIKE DETECTED");
        detector.cooldown_until = Some(Instant::now() + Duration::from_secs(VOLATILITY_COOLDOWN_SECONDS));
        return true;
    }

    return false;
}


