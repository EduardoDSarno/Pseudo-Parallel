import type { Coin, PriceDirection } from "./common.js";

export type { Coin, PriceDirection } from "./common.js";

/* Fired manual price alerts received from the Rust engine */

export type OutgoingManualPriceAlert = {
  type: "manual_price";
  coin: Coin;
  trigger_price: number;
  direction: PriceDirection;
  current_price: number;
};

export type OutgoingAlert = OutgoingManualPriceAlert;
