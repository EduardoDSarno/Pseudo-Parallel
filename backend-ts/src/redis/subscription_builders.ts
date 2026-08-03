import { Command } from "./redis_constants.js";
import type {
  IncomingPriceSubscription,
  IncomingSubscription,
  IncomingSubscriptionType,
} from "./subscription_types/incoming.js";

/* Wraps a sub_type with command subscribe and keeps the command in one place */
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
