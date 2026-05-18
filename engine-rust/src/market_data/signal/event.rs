use crate::market_data::{signal::price::ManualPriceDirection, types::CandleKey};

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
    pub key: CandleKey,
    pub event: Event,
}

impl Alert {
    pub fn new(key: CandleKey, event: Event) -> Self {
        Alert { key, event }
    }
}
