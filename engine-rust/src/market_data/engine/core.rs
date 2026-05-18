use std::collections::{HashMap, VecDeque};
use crate::market_data::{constans::FIRST_CANDLE_INDEX, hyperliquid::protocols::rest::RestResponse, types::{Candle, CandleKey}};

// THis struct will be responsible for handling Candle data and to store in memory 
// the current data we need
pub struct Engine
{
    pub buffers: HashMap<CandleKey, VecDeque<Candle>>,
    last_seen: HashMap<CandleKey, Candle>,
    max_closed_candles: usize,
}


impl Engine
{
    // Lazy approach to create a new engine simply empty Hashmaps
    pub fn new(max_closed_candles: usize) -> Self
    {
        Engine 
        {
            buffers: HashMap::new(),
            last_seen: HashMap::new(),
            max_closed_candles,
        }
    }

    /* This function has the job of seeding our The candle data with the historical previous max_closed_candles
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

        // We use this so we can get the exact number of candles we need for warm up
        if candles.len() < self.max_closed_candles
        {
            let err = format!(
                "cannot seed engine with {} candles, expected at least {}",
                candles.len(),
                self.max_closed_candles
            );
            tracing::error!(received = candles.len(), expected = self.max_closed_candles, error = %err, "Seed candles failed");
            return Err(err);
        }

        let candle_key = CandleKey::new(
            candles[FIRST_CANDLE_INDEX].coin.clone(),
            candles[FIRST_CANDLE_INDEX].interval.clone(),
        );

        // Using a guard to make sure we just have the exact amount of candles we want
        while candles.len() > self.max_closed_candles
        {
            candles.pop_front();
        }

        // overwrites or creates our data for candles for the respective candle_key
        tracing::info!(coin = ?candle_key.coin, interval = ?candle_key.interval, len = candles.len(), "Candle buffer seeded");
        self.buffers.insert(candle_key, candles);
        Ok(())
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

                if buf.len() > self.max_closed_candles
                {
                    buf.pop_front(); // remove from the front of Circle Queue
                }

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
