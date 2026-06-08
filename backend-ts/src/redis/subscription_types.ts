import type { SubscriptionCommand } from "./redis_constants.js";

/* --- primitives (match engine Coins / Interval serde) --- */

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

/* --- nested structs --- */

export type IncomingAtrRule = {
  type: "atr";
  breakout_ratio: number;
  debug_ratio: number;
};

/* tagged union — mirrors Rust IncomingIndicatorKind; add arms when we add indicators */
export type IncomingIndicatorKind = IncomingAtrRule;

export type IncomingPriceSubscription = {
  type: "price";
  coin: Coin;
  trigger_price: number;
  direction?: PriceDirection; // optional;  engine infers
};

export type IncomingIndicatorSubscription = {
  type: "indicator";
  coin: Coin;
  interval: Interval;
  kind: IncomingIndicatorKind;
};

/* tagged union for subscription types */
export type IncomingSubscriptionType =
  | IncomingPriceSubscription
  | IncomingIndicatorSubscription;

/* top-level message — like Rust IncomingSubscription */
export type IncomingSubscription = {
  command: SubscriptionCommand;
  sub_type: IncomingSubscriptionType;
};