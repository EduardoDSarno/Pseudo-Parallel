use std::{collections::HashSet, fs::File};

use hypersdk::{
    Decimal,
    hypercore::{self, types::Subscription},
};

mod consts;
mod liquidation_calc;
mod utils;

use consts::*;
use liquidation_calc::{LiquidationMap, build_liquidation_map, display_heatmap};
use utils::{EventTypes, handle_ws_messages, load_positions, load_set, write_to_file};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // logs init
    let _ = simple_logger::init_with_level(log::Level::Info);

    // ws cooneciton init
    let client = hypercore::mainnet();
    let mut ws: hypercore::WebSocket = client.websocket();

    let mut addresses: HashSet<String> = HashSet::new();

    // load addresses that were already seen in a previous runs
    if let Ok(existing_file) = File::open(ADDRESS_FILE_PATH) {
        load_set(&existing_file, &mut addresses)?;
    }

    // fetch liquidation data for every known address
    // current limiting number of accounts because of API rate limits
    let positions = load_positions(&addresses, &client, Some(200)).await?;

    // subscribing to bct trades
    ws.subscribe(Subscription::Trades {
        coin: BTC_STR.to_string(),
    });

    // subscribe to Hyperliquid’s mark price
    ws.subscribe(Subscription::ActiveAssetCtx {
        coin: BTC_STR.to_string(),
    });

    log::info!("Subscribed to BTC Trades. Waiting for updates...\n");

    let mut current_price_btc: Option<Decimal>;
    let mut heatmap: Option<LiquidationMap> = None;

    // async to wait for connection and handle the message
    while let Some(event) = handle_ws_messages(&mut ws).await {
        match event {
            EventTypes::Trades(trades) => {
                write_to_file(trades, &mut addresses)?;
            }

            EventTypes::MarkPrice(price) => {
                current_price_btc = Some(price);

                if let Some(map) = heatmap.as_mut() {
                    let diff = (map.current_price - price).abs();

                    if diff > MAX_MOVEMENT_USD_BTC.into() {
                        map.current_price = price;
                        display_heatmap(map);
                    }
                } else {
                    let map = build_liquidation_map(&positions, price);
                    display_heatmap(&map);
                    heatmap = Some(map);
                }
                // Compare with the previous reference price.
                // Publish a snapshot if it moved far enough.
                log::info!("Current BTC market Price {:#?}", current_price_btc);
            }
        }
    }

    Ok(())
}
