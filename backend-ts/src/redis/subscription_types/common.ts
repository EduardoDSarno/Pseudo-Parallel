/* Shared wire primitives — match engine Coins / Interval / ManualPriceDirection serde */

// list of the coins of which the subscription can be made, add as needed
export const Coin = {
  Hype: "HYPE",
  Btc: "BTC",
  Eth: "ETH",
} as const;
export type Coin = (typeof Coin)[keyof typeof Coin];

export const Interval = {
  M1: "1m",
  M5: "5m",
  M15: "15m",
  H1: "1h",
} as const;
export type Interval = (typeof Interval)[keyof typeof Interval];

export const PriceDirection = {
  Above: "above",
  Below: "below",
} as const;
export type PriceDirection = (typeof PriceDirection)[keyof typeof PriceDirection];
