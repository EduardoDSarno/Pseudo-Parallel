use std::{collections::HashSet, fs::File};
use std::io::{BufRead, BufReader, BufWriter, Write};

use hypersdk::hypercore::Trade;


 /* This function tries writing to our file the addresses of the buyer and sellet */
pub fn write_to_file(writer: &mut BufWriter<File>, 
                    addresses: &mut HashSet<String>,
                     trade: Trade ) 
                     -> Result<(), Box<dyn std::error::Error>>
{

    log::info!("Received trade, tx {:?}", trade.hash);
    log::info!("BTC current price: {:?}", trade.px);

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

pub fn load_set(file: &File, set: &mut HashSet<String>)
{

    let reader = BufReader::new(file);
    let lines = reader.lines();

    for line in lines
    {
        match line{
            Ok(line) =>
            {
                set.insert(line);
            }
            Err(e) => 
            {
                println!("error adding line {:?}", e);
            }
        }
    }
}
