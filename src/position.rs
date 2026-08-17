use std::time::SystemTime;

use hypersdk::{Decimal, hypercore::{AssetPosition, ClearinghouseState, PositionData}};

use crate::market::Coin;


struct WhalePosition 
{
    address: String,
    coin: Coin,
    signed_size: Decimal,
    position_usd: Decimal,
    liquidation_price: Decimal,
    updated_at: SystemTime,
}


pub fn get_position_by_coin(states: Vec<AssetPosition>, coin: Coin) -> Option<PositionData>
{
    for state in states
    {
        if state.position.coin == coin.as_hyperliquid_symbol()
        {
            return Some(state.position);
        }
    }
    None
}

