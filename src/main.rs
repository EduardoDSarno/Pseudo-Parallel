use std::{collections::HashSet, fs::{File, OpenOptions}};
use std::io::{BufWriter, Write};
use futures::{StreamExt};
use hypersdk::hypercore::{self, Incoming,types::Subscription, ws::Event};

mod utils;
use utils::{load_set, write_to_file};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::init_with_level(log::Level::Info);

    let mut addresses: HashSet<String> = HashSet::new();

    // load addresses that were already seen in a previous runs
    if let Ok(existing_file) = File::open("data/addresses.txt") {
        load_set(&existing_file, &mut addresses);
    }

    // open data file wiht append access
    let data_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("data/addresses.txt")?;
    let mut writer = BufWriter::new(data_file);

    
    let client = hypercore::mainnet();
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

