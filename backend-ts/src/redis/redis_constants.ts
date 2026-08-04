const SUBSCRIPTION_CHANNEL = "alert_subscriptions";
const ALERTS_FIRED_CHANNEL = "alerts_fired";
const REDIS_HOST = "127.0.0.1";
const REDIS_PORT = 6379;

export const Command = {
  Subscribe: "subscribe",
  Unsubscribe: "unsubscribe",
} as const;

export type SubscriptionCommand = (typeof Command)[keyof typeof Command];

export const SUBSCRIBE = Command.Subscribe;
export const UNSUBSCRIBE = Command.Unsubscribe;

export {
  SUBSCRIPTION_CHANNEL,
  ALERTS_FIRED_CHANNEL,
  REDIS_HOST,
  REDIS_PORT,
};
