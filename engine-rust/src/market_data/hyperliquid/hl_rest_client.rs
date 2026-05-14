use std::error::Error;

use reqwest::Client;

use crate::market_data::{constans::HYPERLIQUID_REST_URL, hyperliquid::protocols::rest::{RestRequest, RestResponse, parse_snapshot_to_candles}};


pub async fn send_single_info_request(request: RestRequest) -> Result<RestResponse, Box<dyn Error>>
{
    let req_client = Client::new();

    tracing::debug!(request = ?request, "Sending Hyperliquid REST request");

    let response = req_client
        .post(HYPERLIQUID_REST_URL)
        .json(&request)
        .send()
        .await
        .inspect_err(|err| tracing::error!(request = ?request, error = %err, "Hyperliquid REST request failed"))?;

    let status = response.status();

    let body = response
        .text()
        .await
        .inspect_err(|err| tracing::error!(request = ?request, status = %status, error = %err, "Could not read Hyperliquid REST body"))?;

    tracing::debug!(
        request = ?request,
        status = %status,
        bytes = body.len(),
        "Received Hyperliquid REST response"
    );

    match_info_response(request, &body)
}

pub async fn send_multiple_info_requests(requests: Vec<RestRequest>) -> Result<Vec<RestResponse>, Box<dyn Error>>
{
    let mut responses: Vec<RestResponse> = Vec::new();

    tracing::info!(requests = requests.len(), "Sending Hyperliquid REST batch");

    for request in requests
    {
        let response = send_single_info_request(request).await?;
        responses.push(response);
    }

    tracing::info!(responses = responses.len(), "Finished Hyperliquid REST batch");
    Ok(responses)
}

fn match_info_response(request: RestRequest, body: &str) -> Result<RestResponse, Box<dyn Error>>
{
    match request
    {
        RestRequest::CandleSnapshot(_) => 
        {
            let candle_data = parse_snapshot_to_candles(body)?;
            tracing::debug!(candles = candle_data.len(), "REST candle snapshot parsed");
            Ok(RestResponse::CandleSnapshot(candle_data))
        }
    }
}
