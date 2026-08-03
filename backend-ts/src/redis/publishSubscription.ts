import { SUBSCRIPTION_CHANNEL } from "./redis_constants.js";
import { redis_alert_client } from "./redis.js";
import {
  subscribeMessage,
  unsubscribeMessage,
} from "./subscription_builders.js";
import type {
  IncomingSubscription,
  IncomingSubscriptionType,
} from "./subscription_types/incoming.js";

/* This function we got we will or incoming subscription that is mapped inside 
    subscription types and match a specfic subscription, it stringfy's it and
    published, the Promise<number> returns represent the async value that will arrive
    in the future with the number of subscribers

    Details: The command (subscribe and unsubscribe) are on the helper functions wrap
    */ 
export async function publishSubscriptionCommand(subscription: IncomingSubscription,
): Promise<number> 
{
  const payload = JSON.stringify(subscription);
  return redis_alert_client.publish(SUBSCRIPTION_CHANNEL, payload);
}

/* subscribeMessage + publishSubscriptionCommand in one call — returns Redis subscriber count */
export async function publishSubscribe(
  sub_type: IncomingSubscriptionType,
): Promise<number> {
  return publishSubscriptionCommand(subscribeMessage(sub_type));
}

/* unsubscribeMessage + publishSubscriptionCommand — same return semantics as publish */
export async function publishUnsubscribe(
  sub_type: IncomingSubscriptionType,
): Promise<number> {
  return publishSubscriptionCommand(unsubscribeMessage(sub_type));
}

