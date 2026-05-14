use std::error::Error;

use reqwest::Client;

use crate::market_data::{constans::HYPERLIQUID_REST_URL, hyperliquid::protocols::rest::{RestRequest, RestResponse, parse_snapshot_to_candles}};


pub async fn send_single_info_request(request: RestRequest) -> Result<RestResponse, Box<dyn Error>>
{
    let req_client = Client::new();

    let response = req_client
    .post(HYPERLIQUID_REST_URL)
    .json(&request)
    .send()
    .await?;

    let body = response.text().await?;

    match_info_response(request, &body)
}

pub async fn send_multiple_info_requests(requests: Vec<RestRequest>) -> Result<Vec<RestResponse>, Box<dyn Error>>
{
    let mut responses: Vec<RestResponse> = Vec::new();

    for request in requests
    {
        let response = send_single_info_request(request).await?;
        responses.push(response);
    }

    Ok(responses)
}

fn match_info_response(request: RestRequest, body: &str) -> Result<RestResponse, Box<dyn Error>>
{
    match request
    {
        RestRequest::CandleSnapshot(_) => 
        {
            let candle_data = parse_snapshot_to_candles(body)?;
            Ok(RestResponse::CandleSnapshot(candle_data))
        }
    }
}