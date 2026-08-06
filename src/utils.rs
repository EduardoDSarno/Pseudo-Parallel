use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
};

use futures::StreamExt;
use hypersdk::hypercore::ws::{Connection, Event};
use hypersdk::hypercore::{ClearinghouseState, HttpClient, Incoming, Trade, WebSocket};
use hypersdk::{Address, Decimal};

use crate::consts::ADDRESS_FILE_PATH;

/* This function writes the new addresses from one WebSocket trade batch. */
pub fn write_to_file(
    trades: Vec<Trade>,
    addresses: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> 

{
    
    let data_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ADDRESS_FILE_PATH)?;
    let mut writer = BufWriter::new(data_file);

    for trade in trades {
        log::info!("Received trade, tx {:?}", trade.hash);
        log::info!("BTC current price: {:?}", trade.px);

        let address_buyer = trade.users[0].to_string();
        let address_seller = trade.users[1].to_string();

        if addresses.insert(address_buyer.clone()) {
            writeln!(writer, "{address_buyer}")?;
        }

        if addresses.insert(address_seller.clone()) {
            writeln!(writer, "{address_seller}")?;
        }
    }

    writer.flush()?;
    Ok(())
}

/* This functions will load the address from the file into the Hashset */
pub fn load_set(file: &File, set: &mut HashSet<String>) -> Result<(), Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let lines = reader.lines();
    let mut count = 0;

    for line in lines {
        match line {
            Ok(line) => {
                set.insert(line);
                count += 1;
            }
            Err(e) => {
                println!("error adding line {:?}", e);
            }
        }
    }
    println!("Load successfull {:?} itens added to set", count);
    Ok(())
}

/* Fetches liquidation data for a known set of addresses and builds the position map */
pub async fn load_positions(
    addresses: &HashSet<String>,
    client: &HttpClient,
) -> Result<HashMap<Address, (Decimal, Option<Decimal>)>, Box<dyn std::error::Error>> {
    let mut positions = HashMap::new();

    for addr in addresses {
        let (size, liq) = get_clearing_house_state(client, addr).await?;
        positions.insert(addr.parse()?, (size, liq));
    }

    Ok(positions)
}

/* Get given a addres the clear house state of that address based on the connection
(client) with hyperliquid, it only accessess from hypercore, not other dexes */
pub async fn get_clearing_house_state(
    client: &HttpClient,
    addr: &str,
) -> Result<(Decimal, Option<Decimal>), Box<dyn std::error::Error>> {
    let user: Address = addr.parse()?;
    // need await because is a http request to get the states
    let account_state = client.clearinghouse_state(user, None).await?;

    // each address has a option of liquidation positon for btc with a size in USD
    // holded by a tuple
    for poisitions in &account_state.asset_positions {
        if poisitions.position.coin == "BTC".to_string() {
            return Ok((poisitions.position.szi, poisitions.position.liquidation_px));
        }
    }

    Ok((Decimal::ZERO, None)) // not found, we don't care
}

pub async fn handle_ws_messages(ws: &mut Connection) -> Option<Vec<Trade>> {
    while let Some(event) = ws.next().await
    // awais for messsage from the stream
    {
        match event {
            Event::Connected => {
                log::info!("Websocket Connected!");
            }
            Event::Disconnected => {
                log::info!("Websocket Disconnected");
            }
            // we store every new addressess coming
            Event::Message(Incoming::Trades(trades)) => {
                return Some(trades);
            }
            Event::Message(_) => {} // for all rest of the messages we don't care about
        }
    }

    None
}
