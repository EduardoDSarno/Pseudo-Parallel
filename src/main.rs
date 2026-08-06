use std::{collections::{HashMap, HashSet}, fs::{File, OpenOptions}};
use std::io::{BufWriter, Write};
use futures::{StreamExt};
use hypersdk::{Address, Decimal, hypercore::{self, ClearinghouseState, HttpClient, Incoming, types::Subscription, ws::Event}, hyperevm::morpho::Client};

mod utils;
mod consts;

use consts::*;
use utils::{load_positions, load_set, write_to_file};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::init_with_level(log::Level::Info);
    let client = hypercore::mainnet();

    let mut addresses: HashSet<String> = HashSet::new();

    // load addresses that were already seen in a previous runs
    if let Ok(existing_file) = File::open(ADDRESS_FILE_PATH)
    {
       load_set(&existing_file, &mut addresses)?;
    }

    // fetch liquidation data for every known address
    let positions = load_positions(&addresses, &client).await?;

    // open data file wiht append access
    let data_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ADDRESS_FILE_PATH)?;
    let mut writer = BufWriter::new(data_file);


    
    let mut ws = client.websocket();

    // subscribing to bct trades
    ws.subscribe(Subscription::Trades {
        coin: "BTC".to_string(),
    });

    log::info!("Subscribed to BTC Trades. Waiting for updates...\n");

    while let Some(event) = ws.next().await
    // awais for messsage from the stream
    {
        match event {
            Event::Connected => {
                log::info!("Websocket Connected!");
            }
            Event::Disconnected => {
                log::info!("Websocket Disconnected")
            }
            Event::Message(Incoming::Trades(trades)) => 
            {
                for trade in trades 
                {
                    write_to_file( &mut writer, &mut addresses, trade)?;
                }
                writer.flush()?; // flusing makes the buffer writes to the file

            }
             Event::Message(_) => {} // for all rest of the messages we don't care about
        }
    }

    Ok(())
}




