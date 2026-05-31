use crate::market_data::{
    signal::price::ManualPriceDirection,
    types::{CandleKey, Coins},
};

/* The alers struct is responsible for storing all the types of alers we have wrapped in one structured that
will cover them all */

/* Those are the types of events that currently can be triggered */
#[derive(Debug)]
pub enum Event {
    AtrBreakout {
        atr: f64,
        live_tr: f64,
        ratio: f64,
        spike_level: u64,
        open_time_ms: u64,
    },
    ManualPriceTriggered {
        trigger_price: f64,
        direction: ManualPriceDirection,
        previous_price: f64,
        current_price: f64,
    },
}
#[derive(Debug)]
pub struct Alert {
    pub coin: Coins,
    pub key: Option<CandleKey>,
    pub event: Event,
}

/* The functions below are responsable for creating one type of alert */
impl Alert {
    pub fn indicator(key: CandleKey, event: Event) -> Self {
        Alert {
            coin: key.coin,
            key: Some(key),
            event,
        }
    }

    pub fn manual_price(coin: Coins, event: Event) -> Self {
        Alert {
            coin,
            key: None,
            event,
        }
    }
}
