/* Shared wire primitives — match engine Coins / ManualPriceDirection serde */

// list of the coins of which the subscription can be made, add as needed
export const Coin = {
  Hype: "HYPE",
  Btc: "BTC",
  Eth: "ETH",
} as const;
export type Coin = (typeof Coin)[keyof typeof Coin];

export const PriceDirection = {
  Above: "above",
  Below: "below",
} as const;
export type PriceDirection = (typeof PriceDirection)[keyof typeof PriceDirection];
