use std::{error::Error, time::{SystemTime, UNIX_EPOCH}};

use crate::market_data::{
    constans::MAX_LENGTH_CANDLE_BUFFER,
    engine::Engine,
    hyperliquid::{
        hl_rest_client::send_multiple_info_requests,
        protocols::rest::{CandleSnapshotRequest, RestRequest},
    },
    types::CandleKey,
};

/* This function seeds the engine with previous REST candles before starting the
   live WebSocket stream, so the engine starts with a hot buffer instead of empty data. */
pub async fn seed_engine_from_rest(engine: &mut Engine, candle_keys: &[CandleKey]) -> Result<(), Box<dyn Error>>
{
    let end_time = current_time_ms()?;
    let mut requests: Vec<RestRequest> = Vec::new();

    for candle_key in candle_keys
    {
        // Each interval needs its own time window to get MAX_LENGTH_CANDLE_BUFFER candles.
        let start_time = end_time - (candle_key.interval.to_ms() * MAX_LENGTH_CANDLE_BUFFER as u64);
        let snapshot_request = CandleSnapshotRequest::new(candle_key.clone(), start_time, end_time)
            .inspect_err(|err| tracing::error!(coin = ?candle_key.coin, interval = ?candle_key.interval, error = %err, "Candle snapshot request failed"))
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

        requests.push(RestRequest::CandleSnapshot(snapshot_request));
    }

    let responses = send_multiple_info_requests(requests).await?;
    engine.seed_from_rest_responses(responses)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    Ok(())
}

// Helper to keep timestamp creation outside the main startup flow.
fn current_time_ms() -> Result<u64, Box<dyn Error>>
{
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as u64)
}
