import type { SubscriptionCommand } from "../redis_constants.js";
import type { Coin, PriceDirection } from "./common.js";

export type { Coin, PriceDirection } from "./common.js";

export type IncomingPriceSubscription = {
  type: "price";
  coin: Coin;
  trigger_price: number;
  direction?: PriceDirection; // optional; engine infers
};

/* Keep the alias so more subscription types can be added without changing callers. */
export type IncomingSubscriptionType = IncomingPriceSubscription;

/* top-level message — like Rust IncomingSubscription */
export type IncomingSubscription = {
  command: SubscriptionCommand;
  sub_type: IncomingSubscriptionType;
};
