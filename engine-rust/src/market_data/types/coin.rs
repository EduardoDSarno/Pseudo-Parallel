use serde::{Deserialize, Serialize};

/* Enumerate coin strings into our hard values */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum Coins
{
    HYPE,
    BTC,
    ETH,
}

/* Implementation to handle conversion */
impl TryFrom<String> for Coins
{
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error>
    {
        match value.as_str()
        {
            "HYPE" => Ok(Coins::HYPE),
            "BTC" => Ok(Coins::BTC),
            "ETH" => Ok(Coins::ETH),
            other => Err(format!("unknown coin: {}", other)),
        }
    }
}
