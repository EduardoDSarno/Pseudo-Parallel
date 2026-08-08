use hypersdk::{Address, Decimal};
use std::collections::{BTreeMap, HashMap};

use crate::consts::{HEATMAP_WINDOW_PERCENT, LIQ_LEVEL_SPACING_USD};

pub struct LiquidationMap {
    pub current_price: Decimal,
    pub gap: Decimal,
    pub levels: BTreeMap<Decimal, Decimal>,
}

pub fn build_liquidation_map(
    positions: &HashMap<Address, (Decimal, Option<Decimal>)>,
    current_price: Decimal,
) -> LiquidationMap {
    // gap in between levels
    let gap = Decimal::from(LIQ_LEVEL_SPACING_USD);

    // Btreemap so we can keep adjusting the head based on price and balence
    // short and longs
    let mut levels: BTreeMap<Decimal, Decimal> = BTreeMap::new();

    // loop through the hashmap
    // if there's a liquidation price extract it
    // otherwsie skip this postion and go to the next
    for (_address, (position_size, liquidation_px)) in positions {
        let Some(liquidation_px) = liquidation_px else {
            continue;
        };

        let bucket = (liquidation_px / gap).floor() * gap;

        // it is the current price of the token in the liquidation event times the size
        // if the event
        let position_size_usd = position_size.abs() * liquidation_px;

        // find level on btree and insert or create an empty one (with default)
        // it gets a mutable reference to value stored in the bucket
        let total_usd = levels.entry(bucket).or_default();

        // updates the stored value
        *total_usd += position_size_usd;
    }

    // Create new map
    LiquidationMap {
        current_price,
        gap,
        levels,
    }
}

/// Displays the existing liquidation levels around the current BTC price.
/// This does not rebuild or modify the map.
pub fn display_heatmap(map: &LiquidationMap) {
    let window_percentage = Decimal::from(HEATMAP_WINDOW_PERCENT) / Decimal::from(100);
    let lower_bound = map.current_price * (Decimal::ONE - window_percentage);
    let upper_bound = map.current_price * (Decimal::ONE + window_percentage);

    println!("\nBTC LIQUIDATION HEATMAP");
    println!("Current price: ${}", map.current_price);
    println!("Bucket spacing: ${}\n", map.gap);
    println!(
        "Visible window: ${lower_bound} to ${upper_bound} (+/-{}%)\n",
        HEATMAP_WINDOW_PERCENT
    );
    println!("{:>14} | {:>20} | REGION", "PRICE LEVEL", "TOTAL USD");
    println!("{:-<14}-+-{:-<20}-+--------", "", "");

    let mut visible_levels = map.levels.range(lower_bound..=upper_bound).rev().peekable();

    if visible_levels.peek().is_none() {
        println!("------ CURRENT BTC: ${} ------", map.current_price);
        println!("No liquidation levels inside the visible window.");
        return;
    }

    let mut current_price_displayed = false;

    // BTreeMap is sorted from low to high, so reverse it for a price ladder.
    for (price_level, total_usd) in visible_levels {
        if !current_price_displayed && *price_level <= map.current_price {
            println!("------ CURRENT BTC: ${} ------", map.current_price);
            current_price_displayed = true;
        }

        let bucket_contains_current_price =
            map.current_price >= *price_level && map.current_price < *price_level + map.gap;

        let region = if bucket_contains_current_price {
            "NEAR/MIXED"
        } else if *price_level > map.current_price {
            "SHORT"
        } else {
            "LONG"
        };

        println!("${:>13} | ${:>19} | {region}", price_level, total_usd);
    }

    // If the current price is below every liquidation level, its marker has
    // not been printed yet and belongs at the bottom of the ladder.
    if !current_price_displayed {
        println!("------ CURRENT BTC: ${} ------", map.current_price);
    }

    println!();
}
