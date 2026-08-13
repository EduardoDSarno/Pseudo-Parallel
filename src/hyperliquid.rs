use std::error::Error;

use hypersdk::hypercore;
use hypersdk::hypercore::{ ws::Event, types::*};
use futures::StreamExt;

/// Subscribes to the active asset context for the given coin
/// Returns the asset context from sdk's AssetContext struct
/// If the websocket closes before receiving the asset context, returns an error
/// If the event is not an active asset context, prints the event
/// auto reconnects to the websocket
pub async fn hl_market_data() -> Result<AssetContext, Box<dyn Error>>
{
    let mut ws = hypercore::mainnet_ws();

    ws.subscribe(Subscription::ActiveAssetCtx { coin: "BTC".into() });

    while let Some(event) = ws.next().await
    {
        match event
        {
            Event::Connected =>{
                println!("Connected to Hyperliquid");
            },
            Event::Disconnected =>
            {
                println!("Disconnected from Hyperliquid");
            },
            // When the active asset context is received
            // Return the asset context from sdk's AssetContext struct
            Event::Message(Incoming::ActiveAssetCtx { ctx, .. }) =>
            {
                return Ok(ctx);
            },
            _ =>
            {
                println!("Event: {:?}", event);
            },
        }
    }
    Err("Websocket closed before receiving asset context".into())
}