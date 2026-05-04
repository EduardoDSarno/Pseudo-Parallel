use std::collections::{HashMap, VecDeque};
use crate::market_data::{constans::*, indicators::atr::{calculate_average_true_range, calculate_true_range}, signal::event::{BreakoutAlert, Event}, types::candle::{Candle, CandleKey}};

// THis struct will be responsible for handling Candle data and to store in memory 
// the current data we need
pub struct Engine
{
    buffers: HashMap<CandleKey, VecDeque<Candle>>,
    last_seen: HashMap<CandleKey, Candle>
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
                let to_push = last.clone();
                // entry key returns a Entry enum that checks if the key is Ocuppied or Free
                // if free runs new::VecDeque if not it will
                // So buf will be bascially be our mut vecDeque (queue)
                let buf = self.buffers.entry(candle_key.clone())
                .or_insert_with(VecDeque::new);

                // push last because canlde is the one that just started
                // to its map, clone so it does not takes ownership
                tracing::debug!(coin = ?to_push.coin, interval = ?to_push.interval, open_time = to_push.open_time_ms, "Candle closed and added to buffer");
                buf.push_back(to_push);

                if buf.len() > MAX_LENGTH_CANDLE_BUFFER 
                {
                    buf.pop_front(); // remove from the front of Circle Queue
                }

               
                return Some(candle);
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


    pub fn evaluate_breakout(&self, candle: &Candle) -> Option<BreakoutAlert>
    {
        let key = CandleKey::create_key_from_candle(candle);
        let buf = self.buffers.get(&key)?;

        if buf.len() < MAX_LENGTH_CANDLE_BUFFER 
        {
            tracing::debug!(len = buf.len(), max = MAX_LENGTH_CANDLE_BUFFER, "Buffer warming up");
            return None;
        }
    
        // Buffer is full — calculate indicators
        
        // calculate ATR using just valid candles (LAST_CANDLE_INDEX) lenght
        let atr_input: Vec<Candle> = buf.iter().take(LAST_CANDLE_INDEX).cloned().collect();
        let atr = calculate_average_true_range(&atr_input)?;
        // Calculate TR from last candle
        let tr_last = calculate_true_range(&buf[SECOND_TO_LAST_CANDLE_INDEX], &buf[LAST_CANDLE_INDEX]);

        tracing::debug!(coin = ?key.coin, interval = ?key.interval, tr_last = tr_last, atr = atr, ratio = tr_last / atr, "ATR evaluated");

        // detect spike 
        if tr_last > ATR_BREAKOUT_RATIO * atr
        {
            // fire break 
            let event = Event::ATR { prev_atr: atr, breakout_atr: tr_last };
            let breakout_alert = BreakoutAlert::new(key, event);
            
            return Some(breakout_alert);
        }
    
        // Returst None if Buffer is not filled
        None
    }

}

