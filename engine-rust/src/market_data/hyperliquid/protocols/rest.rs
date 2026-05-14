use serde::{Serialize, Deserialize};

use crate::market_data::{constans::MAX_LENGTH_CANDLE_BUFFER, hyperliquid::protocols::data_models::candle::CandleHL, types::{Candle, CandleKey}};

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
    pub fn new(candle_key: CandleKey, start_time: u64, end_time: u64) -> Result<CandleSnapshotRequest, String>
    {
        if end_time <= start_time
        {
            return Err("end_time must be greater than start_time".to_string());
        }

        let interval_ms = candle_key.interval.to_ms();
        let minimum_window_ms = interval_ms * MAX_LENGTH_CANDLE_BUFFER as u64;
        let requested_window_ms = end_time - start_time;

        if requested_window_ms < minimum_window_ms
        {
            return Err(format!(
                "candle snapshot window is too small: requested {} ms, minimum {} ms",
                requested_window_ms,
                minimum_window_ms
            ));
        }

        Ok(CandleSnapshotRequest
        {
            candle_key: candle_key,
            start_time: start_time,
            end_time  : end_time
        })
    }
}

pub enum RestResponse 
{
    CandleSnapshot(Vec<Candle>),
}

/* This function has the job of receving the INbound message from Hyperliquid and converting the HL_Candles in Json
    to our correct typed candles so we can use the data */
pub fn parse_snapshot_to_candles(json: &str) -> Result<Vec<Candle>, Box<dyn std::error::Error>>
{
    let candles_hl:Vec<CandleHL> = serde_json::from_str(json)
        .inspect_err(|err| tracing::error!(error = %err, "Could not parse REST candle snapshot"))?;
    let mut candles : Vec<Candle> = Vec::new();

    for candle_hl in candles_hl 
    {
        let candle = Candle::try_from(candle_hl)
            .inspect_err(|err| tracing::error!(error = %err, "Could not convert REST candle"))?;
        candles.push(candle);
    }

    Ok(candles)
}
