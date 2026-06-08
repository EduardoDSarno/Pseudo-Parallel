import { Command } from "./redis_constants.js";
import type {
  IncomingIndicatorSubscription,
  IncomingPriceSubscription,
  IncomingSubscription,
  IncomingSubscriptionType,
} from "./subscription_types.js";

/* Wraps a sub_type with command subscribe — caller only passes price or indicator
   payload; we set Command.Subscribe here so it stays in one place */
export function subscribeMessage(
  sub_type: IncomingSubscriptionType,
): IncomingSubscription {
  return { command: Command.Subscribe, sub_type };
}

/* Same as subscribeMessage but command unsubscribe — sub_type must match what was
   subscribed (coin, trigger, direction rules) or the engine won't find the book entry */
export function unsubscribeMessage(
  sub_type: IncomingSubscriptionType,
): IncomingSubscription {
  return { command: Command.Unsubscribe, sub_type };
}

/* Builds an price sub_type tagged union arm helper to ommit type */
export function priceSubType(
  sub_type: Omit<IncomingPriceSubscription, "type">,
): IncomingPriceSubscription {
  return { type: "price", ...sub_type };
}

/* Builds an indicator sub_type tagged union arm helper to ommit type */
export function indicatorSubType(
  sub_type: Omit<IncomingIndicatorSubscription, "type">,
): IncomingIndicatorSubscription {
  return { type: "indicator", ...sub_type };
}
