import type { SubscriptionCommand } from "../redis_constants.js";
import type { Coin, Interval, PriceDirection } from "./common.js";

export type { Coin, Interval, PriceDirection } from "./common.js";

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
  direction?: PriceDirection; // optional; engine infers
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
