use std::{collections::HashSet, fs::{self, File}, writeln};
use std::io::{BufWriter, Write};
use futures::StreamExt;
use hypersdk::hypercore::{self, Incoming, Trade, types::Subscription, ws::Event};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::init_with_level(log::Level::Info);

    let mut writer = BufWriter::new(File::create("data/addresses.txt")?);

    let mut  addresses: HashSet<String> = HashSet::new();

    
    let client = hypercore::mainnet();
    let mut ws = client.websocket();

    // subscribing to bct trades
    ws.subscribe(Subscription::Trades {
        coin: "BTC".to_string(),
    });

    log::info!("Subscribed to BTC 1m candles. Waiting for updates...\n");

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
            }
             Event::Message(_) => {} // for all rest of the messages we don't care about
        }
    }

    Ok(())
}

/* This function tries writing to our file the addresses of the buyer and sellet */
pub fn write_to_file(writer: &mut BufWriter<File>, 
                    addresses: &mut HashSet<String>,
                     trade: Trade ) 
                     -> Result<(), Box<dyn std::error::Error>>
{

    log::info!("Received trade: {:?}", trade);

                    let address_buyer:  String = trade.users[0].to_string();
                    let address_seller: String = trade.users[1].to_string();

                    // address.insert returns true when address is not in the set
                    // so we don't need a separate contains
                    // wrinteln is a macro to append to the file
                    if addresses.insert(address_buyer.clone()) 
                    {
                        writeln!(writer, "{address_buyer}")?;
                    }
                    
                    if addresses.insert(address_seller.clone())
                    {
                        writeln!(writer, "{address_seller}")?;
                    }
    Ok(())
}
