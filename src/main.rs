use std::{collections::HashSet, fs::File};

use hypersdk::hypercore::{self, types::Subscription};

mod consts;
mod utils;
mod liquidation_calc;

use consts::*;
use utils::{handle_ws_messages, load_positions, load_set, write_to_file};

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
    let positions = load_positions(&addresses, &client).await?;

    // subscribing to bct trades
    ws.subscribe(Subscription::Trades {
        coin: BTC_STR.to_string(),
    });

    log::info!("Subscribed to BTC Trades. Waiting for updates...\n");

    // async to wait for connection and handle the message
    while let Some(trades) = handle_ws_messages(&mut ws).await {
        write_to_file(trades, &mut addresses)?;
    }
    
    Ok(())
}
