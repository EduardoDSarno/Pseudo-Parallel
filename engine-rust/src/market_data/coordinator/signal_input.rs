use crate::market_data::coordinator::candle_ingest::IngestedCandleSnapshot;

/* Signal input types we can receive */
pub enum SignalInput {
    Candle(IngestedCandleSnapshot),
}
