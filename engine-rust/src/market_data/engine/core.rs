use std::collections::{HashMap, VecDeque};
use crate::market_data::{constans::*, hyperliquid::protocols::rest::RestResponse, indicators::atr::{calculate_average_true_range, calculate_true_range}, signal::event::{BreakoutAlert, Event}, types::{Candle, CandleKey}};

// THis struct will be responsible for handling Candle data and to store in memory 
// the current data we need
pub struct Engine
{
    pub(super) buffers: HashMap<CandleKey, VecDeque<Candle>>,
    last_seen: HashMap<CandleKey, Candle>,
    live_alerts: HashMap<CandleKey, LiveAlertState>,
}

#[derive(Debug, Clone)]
struct LiveAlertState
{
    open_time_ms: u64,
    last_spike_level: u64,
}

impl Engine
{
    // Lazy approach to create a new engine simply empty Hashmaps
    pub fn new() -> Self
    {
        Engine 
        {
            buffers: HashMap::new(),
            last_seen: HashMap::new(),
            live_alerts: HashMap::new(),
        }
    }

    /*  This function will receive a Candle, and the engine. it will create a Candle Key
     from it and it will insert it self on the hashmap when new candle is found
     else it will just update the last seen 
    */
    pub fn handle_candle(&mut self, candle: Candle) -> Option<Candle>
    {

        let candle_key = CandleKey::new(candle.coin.clone(), candle.interval.clone());


        // if let we get Some (to unwrap option) candle with the key
        if let Some(last) = self.last_seen.get(&candle_key) 
        {
            // we check if the time is different 
            if last.open_time_ms != candle.open_time_ms 
            {
                // add another variable here to be able to pass inside push_back
                // without breaking the imutability of self by using last
                let closed = last.clone();
                // entry key returns a Entry enum that checks if the key is Ocuppied or Free
                // if free runs new::VecDeque if not it will
                // So buf will be bascially be our mut vecDeque (queue)
                let buf = self.buffers.entry(candle_key.clone())
                .or_insert_with(VecDeque::new);

                // push last because canlde is the one that just started
                // to its map, clone so it does not takes ownership
                tracing::debug!(coin = ?closed.coin, interval = ?closed.interval, open_time = closed.open_time_ms, "Candle closed and added to buffer");
                buf.push_back(closed.clone());

                if buf.len() > MAX_LENGTH_CANDLE_BUFFER 
                {
                    buf.pop_front(); // remove from the front of Circle Queue
                }

                self.live_alerts.remove(&candle_key);
                self.last_seen.insert(candle_key, candle);
                return Some(closed);
            }
            else 
            {
                // If now new candle, just update last seen
                self.last_seen.insert(candle_key, candle);
                return None    
            }
        }
        else 
        {
            // If no candle match add, return None
            self.last_seen.insert(candle_key, candle);
            return None;
        }
    }


    /*
        This function will check the currenlty forming live candle against an ATR baseline build only from
        closed candles it will return Some(BreakoutAlern) if a spike is detected otherwise is a No. It will take a
        &mut self because it needs to update the spike levels is already did so there's not repetiton
         */
    pub fn evaluate_live_breakout(&mut self, candle: &Candle) -> Option<BreakoutAlert>
    {

        // Create Key and find the correct buffer
        let key = CandleKey::create_key_from_candle(candle);
        let buf = self.buffers.get(&key)?;

        // Check if we can already calculate ATR
        if buf.len() < MAX_LENGTH_CANDLE_BUFFER 
        {
            tracing::debug!(coin = ?key.coin, interval = ?key.interval, len = buf.len(), max = MAX_LENGTH_CANDLE_BUFFER, "Buffer warming up");
            return None;
        }
    
        // Buffer is full, so calculate live candle against closed candle ATR
        let atr_input: Vec<Candle> = buf.iter().cloned().collect();
        let atr = calculate_average_true_range(&atr_input)?;
        if atr <= MIN_VALID_ATR
        {
            tracing::warn!(coin = ?key.coin, interval = ?key.interval, atr = atr, "ATR is not valid for live breakout");
            return None;
        }

        // Calculating current true range
        let latest_closed = buf.back()?;
        let live_tr = calculate_true_range(latest_closed, candle);
        let ratio = live_tr / atr;
        let spike_level = (ratio / ATR_BREAKOUT_RATIO).floor() as u64;

        if ratio >= ATR_BREAKOUT_RATIO * LIVE_ATR_DEBUG_RATIO
        {
            tracing::debug!(coin = ?key.coin, interval = ?key.interval, open_time = candle.open_time_ms, live_tr = live_tr, atr = atr, ratio = ratio, spike_level = spike_level, "Live ATR evaluated");
        }

        if spike_level == NO_SPIKE_LEVEL
        {
            return None;
        }

        // Look inside self.live_alerts for this coin/interval.
        // If state already exists, give me a mutable reference to it.
        // If it does not exist, create a new LiveAlertState.
        let state = self.live_alerts.entry(key.clone()).or_insert(LiveAlertState
        {
            open_time_ms: candle.open_time_ms,
            last_spike_level: NO_SPIKE_LEVEL,
        });

        //If the candle changed, reset the alert state.

        if state.open_time_ms != candle.open_time_ms
        {
            state.open_time_ms = candle.open_time_ms;
            state.last_spike_level = NO_SPIKE_LEVEL;
        }

        if spike_level <= state.last_spike_level
        {
            return None;
        }

        state.last_spike_level = spike_level;
        let event = Event::ATR
        {
            atr,
            live_tr,
            ratio,
            spike_level,
            open_time_ms: candle.open_time_ms,
        };
        let breakout_alert = BreakoutAlert::new(key, event);

        Some(breakout_alert)
    }

    /* This function has the job of seeding our The candle data with the historical previous  MAX_LENGTH_CANDLE_BUFFER
        candles, so it becomes a hot start insated of a cold one*/
    pub fn seed_candles(&mut self, mut candles: VecDeque<Candle>) -> Result<(), String>
    {
        // Data not passed
        if candles.is_empty()
        {
            let err = "cannot seed engine with empty candle buffer".to_string();
            tracing::error!(error = %err, "Seed candles failed");
            return Err(err);
        }

        // We use this so we can get the exact number of candles we need
        if candles.len() < MAX_LENGTH_CANDLE_BUFFER
        {
            let err = format!(
                "cannot seed engine with {} candles, expected at least {}",
                candles.len(),
                MAX_LENGTH_CANDLE_BUFFER
            );
            tracing::error!(received = candles.len(), expected = MAX_LENGTH_CANDLE_BUFFER, error = %err, "Seed candles failed");
            return Err(err);
        }

        let candle_key = CandleKey::new(
            candles[FIRST_CANDLE_INDEX].coin.clone(),
            candles[FIRST_CANDLE_INDEX].interval.clone(),
        );

        // Using a guard to make sure we just have the exact amount of candles we want
        while candles.len() > MAX_LENGTH_CANDLE_BUFFER
        {
            candles.pop_front();
        }

        // overwrites or creates our data for candles for the respective candle_key
        tracing::info!(coin = ?candle_key.coin, interval = ?candle_key.interval, len = candles.len(), "Candle buffer seeded");
        self.buffers.insert(candle_key, candles);
        Ok(())
    }

    /* This is a wrapper for seed candles to handle a vector of responsed insated of one only */
    pub fn seed_from_rest_responses(&mut self, responses: Vec<RestResponse>) -> Result<(), String>
    {
        tracing::info!(responses = responses.len(), "Seeding engine from REST responses");

        for response in responses
        {
            match response
            {
                RestResponse::CandleSnapshot(candles) =>
                {
                    self.seed_candles(VecDeque::from(candles))?;
                }
            }
        }

        Ok(())
    }
}
