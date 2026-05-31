use std::{
    error::Error,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::market_data::{
    constans::{
        REST_SEED_MAX_ATTEMPTS, REST_SEED_RETRY_INITIAL_MS, WS_RECONNECT_BACKOFF_MULTIPLIER,
        WS_RECONNECT_MAX_MS,
    },
    hyperliquid::{
        hl_rest_client::send_multiple_info_requests,
        protocols::rest::{CandleSnapshotRequest, RestRequest},
    },
    runtime::MarketDataRuntime,
    types::CandleKey,
};

/* Seeds the engine from Hyperliquid REST before the live WebSocket starts.
We need a full closed-candle buffer per stream so indicators (ATR) can run on the first
live tick — not a cold start. Retries handle transient API/network failures; if all
attempts fail we abort startup (main never opens WS) instead of running with bad data. */
pub async fn seed_engine_from_rest(
    runtime: &mut MarketDataRuntime,
    candle_keys: &[CandleKey],
) -> Result<(), Box<dyn Error>> {
    let max_closed_candles = runtime.max_closed_candles();
    let mut backoff = Duration::from_millis(REST_SEED_RETRY_INITIAL_MS);
    let mut last_error: Option<Box<dyn Error>> = None;

    for attempt in 1..=REST_SEED_MAX_ATTEMPTS {
        // fresh end_time each attempt so the snapshot window moves forward on retry
        let end_time = current_time_ms()?;
        let requests = build_seed_requests(candle_keys, end_time, max_closed_candles)?;

        tracing::info!(
            attempt,
            max_attempts = REST_SEED_MAX_ATTEMPTS,
            streams = candle_keys.len(),
            end_time,
            "REST seed attempt"
        );

        match try_seed_candle_once(runtime, &requests, candle_keys).await {
            Ok(()) => {
                tracing::info!(attempt, "REST seed succeeded");
                return Ok(());
            }
            Err(err) => {
                tracing::warn!(
                    attempt,
                    max_attempts = REST_SEED_MAX_ATTEMPTS,
                    error = %err,
                    "REST seed attempt failed"
                );
                last_error = Some(err);
                if attempt < REST_SEED_MAX_ATTEMPTS {
                    // wait before retry — same backoff idea as WS reconnect
                    tokio::time::sleep(backoff).await;
                    backoff = next_backoff(backoff);
                }
            }
        }
    }

    // all attempts failed — do not start live stream with partial or empty buffers
    let err = last_error.unwrap_or_else(|| "REST seed failed with no error detail".into());
    tracing::error!(
        max_attempts = REST_SEED_MAX_ATTEMPTS,
        error = %err,
        "REST seed exhausted all attempts"
    );
    Err(err)
}

/* One full seed try: fetch all streams, load into engine, then audit every key.
Fails if REST errors, buffer too short, or verify finds missing buffer / last_seen. */
async fn try_seed_candle_once(
    runtime: &mut MarketDataRuntime,
    requests: &[RestRequest],
    candle_keys: &[CandleKey],
) -> Result<(), Box<dyn Error>> {
    let responses = send_multiple_info_requests(requests.to_vec()).await?;
    tracing::info!(responses = responses.len(), "REST seed responses received");

    // seed_candles also sets last_seen from the latest REST bar (see engine/seed.rs)
    runtime
        .seed_from_rest_responses(responses)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    // per-stream check: closed_len == max_closed_candles and last_seen present
    runtime
        .verify_seeded_keys(candle_keys)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    tracing::info!("REST seed loaded into engine");
    Ok(())
}

/* Builds one REST candleSnapshot request per subscribed stream.
Window length = interval_ms * max_closed_candles so HL returns enough bars for warmup. */
fn build_seed_requests(
    candle_keys: &[CandleKey],
    end_time: u64,
    max_closed_candles: usize,
) -> Result<Vec<RestRequest>, Box<dyn Error>> {
    let mut requests: Vec<RestRequest> = Vec::new();

    for candle_key in candle_keys {
        let start_time = end_time - (candle_key.interval.to_ms() * max_closed_candles as u64);
        tracing::debug!(
            coin = ?candle_key.coin,
            interval = ?candle_key.interval,
            start_time,
            end_time,
            "Building candle snapshot request"
        );

        let snapshot_request = CandleSnapshotRequest::new(
            candle_key.clone(),
            start_time,
            end_time,
            max_closed_candles,
        )
        .inspect_err(|err| {
            tracing::error!(
                coin = ?candle_key.coin,
                interval = ?candle_key.interval,
                error = %err,
                "Candle snapshot request failed"
            )
        })
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

        requests.push(RestRequest::CandleSnapshot(snapshot_request));
    }

    Ok(requests)
}

/* Double wait between seed retries, capped — avoids hammering REST on outage */
fn next_backoff(current: Duration) -> Duration {
    let doubled = current.saturating_mul(WS_RECONNECT_BACKOFF_MULTIPLIER);
    let max = Duration::from_millis(WS_RECONNECT_MAX_MS);
    if doubled > max {
        max
    } else {
        doubled
    }
}

fn current_time_ms() -> Result<u64, Box<dyn Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64)
}
