import { Coin, PriceDirection } from "./common.js";

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

const VALID_COINS: readonly string[] = Object.values(Coin);
const VALID_DIRECTIONS: readonly string[] = Object.values(PriceDirection);

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

/* Runtime check for JSON.parse output — JSON.parse only proves the string was
valid JSON, not that it matches OutgoingAlert's shape. Engine bugs, version
skew, or corrupted payloads could otherwise flow through as a wrongly-typed
object and break whatever reads it downstream. */
export function isOutgoingAlert(value: unknown): value is OutgoingAlert {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;

  return (
    candidate.type === "manual_price" &&
    typeof candidate.coin === "string" &&
    VALID_COINS.includes(candidate.coin) &&
    isFiniteNumber(candidate.trigger_price) &&
    typeof candidate.direction === "string" &&
    VALID_DIRECTIONS.includes(candidate.direction) &&
    isFiniteNumber(candidate.current_price)
  );
}
