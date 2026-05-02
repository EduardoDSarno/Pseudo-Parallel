use std::collections::{HashMap, VecDeque};
use crate::market_data::{constans::{MAX_LENGTH_CANDLE_BUFFER}, types::candle::{Candle, CandleKey}};

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
    pub fn handle_candle(&mut self, candle: Candle)
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
                println!("New Candle Added: {:#?}", &to_push);
                buf.push_back(to_push);

                if buf.len() > MAX_LENGTH_CANDLE_BUFFER 
                {
                    buf.pop_front(); // remove from the front of Circle Queue
                }
            }
        }
        // if does not exist insert, is does exits replace
        self.last_seen.insert(candle_key, candle);
    }
}

