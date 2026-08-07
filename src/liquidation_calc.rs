use std::collections::{BTreeMap, HashMap};
use hypersdk::{Address, Decimal};

use crate::consts::LIQ_LEVEL_SPACING_USD;

mod consts 
{
    pub const LIQ_LEVEL_SPACING_USD: u64 = 1;
}


pub struct LiquidationMap
{
    pub current_price: Decimal,
    pub gap: Decimal,
    pub levels: BTreeMap<Decimal, LiquidationLevel>,
}

#[derive(Default)]
pub struct LiquidationLevel
{
    pub size_usd: Decimal,
    pub addresses_count: usize,
}



pub fn build_liquidation_map(positions: HashMap<Address, (Decimal, Option<Decimal>)>,
                                current_price: Decimal) -> LiquidationMap 
{
    // gap in between levels
    let gap = Decimal::from(LIQ_LEVEL_SPACING_USD);

    // Btreemap so we can keep adjusting the head based on price and balence
    // short and longs
    let mut levels: BTreeMap<Decimal, LiquidationLevel> = BTreeMap::new();



    // loop through the hashmap
    // if there's a liquidation price extract it
    // otherwsie skip this postion and go to the next
    for (_address , (position_size, liquidation_px)) in positions
    {
        let Some(liquidation_px) = liquidation_px else 
        {
            continue;    
        };
    

        let bucket = (liquidation_px / gap).floor() * gap;

        // it is the current price of the token in the liquidation event times the size
        // if the event
        let position_size_usd = position_size.abs() * liquidation_px;


        // find level on btree and insert or create an empty one (with default)
        let level: &mut LiquidationLevel = levels
        .entry(bucket)
        .or_default();

        level.size_usd        += position_size_usd;
        level.addresses_count += 1;
    };

    // Create new map
    LiquidationMap 
    { 
        current_price,
        gap,
        levels
    }
}
