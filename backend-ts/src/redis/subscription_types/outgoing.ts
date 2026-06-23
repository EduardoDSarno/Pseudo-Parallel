import type { Coin, Interval, PriceDirection } from "./common.js";

export type { Coin, Interval, PriceDirection } from "./common.js";

/* Fired alerts on alerts_fired — mirrors engine Event / Alert wire shape */

export type OutgoingManualPriceAlert = {
  type: "manual_price";
  coin: Coin;
  trigger_price: number;
  direction: PriceDirection;
  previous_price: number;
  current_price: number;
};

export type OutgoingAtrBreakoutAlert = {
  type: "atr_breakout";
  coin: Coin;
  interval: Interval;
  indicator_rule_id: number;
  atr: number;
  live_tr: number;
  ratio: number;
  spike_level: number;
  open_time_ms: number;
};

export type OutgoingAlert =
  | OutgoingManualPriceAlert
  | OutgoingAtrBreakoutAlert;
