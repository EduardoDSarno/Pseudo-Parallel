use serde::{Serialize, Deserialize};

use crate::market_data::{hyperliquid::protocols::data_models::candle::{CandleHL}, types::candle::{Candle, CandleKey}};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", content = "req", rename_all = "camelCase")]
pub enum RestRequest 
{
    CandleSnapshot(CandleSnapshotRequest),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandleSnapshotRequest 
{
    #[serde(flatten)]
    pub candle_key: CandleKey,
    pub start_time: u64,
    pub end_time: u64,
}


impl CandleSnapshotRequest
{
    pub fn new(candle_key: CandleKey, start_time: u64, end_time: u64) -> CandleSnapshotRequest
    {
        let req = CandleSnapshotRequest
        {
            candle_key: candle_key,
            start_time: start_time,
            end_time  : end_time
        };
        req
    }
}

// Alaias for getting our Rest Response
pub type CandleSnapshotResponse = Vec<CandleHL>;

/* This function has the job of receving the INbound message from Hyperliquid and converting the HL_Candles in Json
    to our correct typed candles so we can use the data */
pub fn parse_snapshot_to_candles(json: &str) -> Result<Vec<Candle>, Box<dyn std::error::Error>>
{
    let candles_hl:CandleSnapshotResponse = serde_json::from_str(json)?;
    let mut candles : Vec<Candle> = Vec::new();

    for candle_hl in candles_hl 
    {
        let candle = Candle::try_from(candle_hl)?;
        candles.push(candle);
    }

    Ok(candles)
}

