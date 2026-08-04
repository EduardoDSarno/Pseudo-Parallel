import assert from "node:assert/strict";
import { after, before, test } from "node:test";

import { Redis } from "ioredis";

import { publishSubscribe } from "../../src/redis/publishSubscription.js";
import { redis_alert_client, redis_alert_subscriber } from "../../src/redis/redis.js";
import {
  ALERTS_FIRED_CHANNEL,
  REDIS_HOST,
  REDIS_PORT,
  SUBSCRIPTION_CHANNEL,
} from "../../src/redis/redis_constants.js";
import { subscribeAlerts } from "../../src/redis/subscribeAlerts.js";
import { priceSubType } from "../../src/redis/subscription_builders.js";
import { Coin } from "../../src/redis/subscription_types/common.js";
import type { OutgoingAlert } from "../../src/redis/subscription_types/outgoing.js";

/* Integration test for the Redis wire contract in both directions:
   - TS -> engine: publishSubscribe puts the JSON shape the engine's incoming.rs expects
     onto SUBSCRIPTION_CHANNEL.
   - engine -> TS: subscribeAlerts correctly receives and validates fired-alert JSON on
     ALERTS_FIRED_CHANNEL — same shape the engine's outgoing.rs produces.

   This requires a real Redis instance reachable at REDIS_HOST:REDIS_PORT. There's no
   real engine involved — a real price crossing depends on live market data, which
   isn't deterministic enough to test against. Instead, we play the engine's role
   directly: publish alert JSON exactly as outgoing.rs would produce it, and assert
   the TS side handles it correctly. */

let testPublisher: Redis;
const receivedAlerts: OutgoingAlert[] = [];

before(async () => {
  testPublisher = new Redis({ host: REDIS_HOST, port: REDIS_PORT });

  // subscribeAlerts awaits its own redis .subscribe() call before returning, so once
  // this resolves we're actually listening — no race with the publishes below.
  await subscribeAlerts((alert) => {
    receivedAlerts.push(alert);
  });
});

after(() => {
  testPublisher.disconnect();
  redis_alert_subscriber.disconnect();
  redis_alert_client.disconnect();
});

/* Polls receivedAlerts until predicate matches or timeoutMs elapses — pub/sub
delivery isn't instant, so we wait-with-timeout instead of guessing a fixed delay. */
async function waitForAlert(
  predicate: (alert: OutgoingAlert) => boolean,
  timeoutMs = 2000,
): Promise<OutgoingAlert> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const found = receivedAlerts.find(predicate);
    if (found) return found;
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
  throw new Error(`timed out waiting for alert matching predicate after ${timeoutMs}ms`);
}

test("publishSubscribe puts a wire-correct payload on the subscription channel", async () => {
  const rawSubscriber = new Redis({ host: REDIS_HOST, port: REDIS_PORT });
  const received = new Promise<string>((resolve) => {
    rawSubscriber.on("message", (channel, message) => {
      if (channel === SUBSCRIPTION_CHANNEL) resolve(message);
    });
  });
  await rawSubscriber.subscribe(SUBSCRIPTION_CHANNEL);

  await publishSubscribe(priceSubType({ coin: Coin.Hype, trigger_price: 70 }));

  const raw = await Promise.race([
    received,
    new Promise<string>((_, reject) =>
      setTimeout(() => reject(new Error("timed out waiting for subscription publish")), 2000),
    ),
  ]);
  rawSubscriber.disconnect();

  const parsed = JSON.parse(raw);
  assert.equal(parsed.command, "subscribe");
  assert.equal(parsed.sub_type.type, "price");
  assert.equal(parsed.sub_type.coin, "HYPE");
  assert.equal(parsed.sub_type.trigger_price, 70);
});

test("subscribeAlerts receives and parses a fired alert published on alerts_fired", async () => {
  const fakeAlert = {
    type: "manual_price",
    coin: "HYPE",
    trigger_price: 71,
    direction: "above",
    current_price: 71.2,
  };

  await testPublisher.publish(ALERTS_FIRED_CHANNEL, JSON.stringify(fakeAlert));

  const received = await waitForAlert((alert) => alert.trigger_price === 71);

  assert.deepEqual(received, fakeAlert);
});

test("malformed alert payloads are dropped instead of reaching the callback", async () => {
  // neither of these matches OutgoingAlert's shape — isOutgoingAlert should reject both
  await testPublisher.publish(ALERTS_FIRED_CHANNEL, "not json");
  await testPublisher.publish(
    ALERTS_FIRED_CHANNEL,
    JSON.stringify({ type: "manual_price", coin: "HYPE" }), // missing required fields
  );

  // a sentinel published after the bad payloads — if it arrives, we know the pipe
  // kept working and wasn't stuck on the bad messages ahead of it
  const sentinel = {
    type: "manual_price",
    coin: "HYPE",
    trigger_price: 999,
    direction: "below",
    current_price: 998.5,
  };
  await testPublisher.publish(ALERTS_FIRED_CHANNEL, JSON.stringify(sentinel));

  const received = await waitForAlert((alert) => alert.trigger_price === 999);
  assert.deepEqual(received, sentinel);

  const badPayloadDelivered = receivedAlerts.some(
    (alert) => alert.trigger_price !== 71 && alert.trigger_price !== 999,
  );
  assert.equal(badPayloadDelivered, false);
});