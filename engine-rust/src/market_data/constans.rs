use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum COINS{
    HYPE,
    BTC,
    ETH,
}