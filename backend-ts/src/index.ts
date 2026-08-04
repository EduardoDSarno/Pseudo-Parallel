import { publishSubscribe } from "./redis/publishSubscription.js";
import { priceSubType } from "./redis/subscription_builders.js";
import { Coin } from "./redis/subscription_types/common.js";
import { subscribeAlerts } from "./redis/subscribeAlerts.js";
import { removeAlert } from "./web/alertStore.js";
import { ALERT_SERVER_PORT, broadcastAlertFired, startAlertServer } from "./web/server.js";

async function main(): Promise<void> {
  // Subscribe to alerts, log them, drop them from the local store (manual price
  // alerts are one-shot, so the engine already removed it), and push to any
  // open browser tabs so they can notify.
  await subscribeAlerts((alert) =>
  {
      console.log("Alert received:", alert);
      removeAlert({ coin: alert.coin, direction: alert.direction, trigger_price: alert.trigger_price });
      broadcastAlertFired(alert);
  });

  startAlertServer();
  console.log(`Alert web interface: http://localhost:${ALERT_SERVER_PORT}`);

  if (process.env.SUBSCRIPTION_SMOKE !== "1") {
    console.log("Backend started. Set SUBSCRIPTION_SMOKE=1 to publish a test subscription.");
    return;
  }

  /* Dev-only smoke: publish one subscription so engine can receive it from redis. */
  const subscriberCount = await publishSubscribe(
    priceSubType({
      coin: Coin.Hype,
      trigger_price: 70,
    }),
  );
  console.log(`Subscription smoke published to ${subscriberCount} Redis subscriber(s).`);
}

main().catch((err: unknown) => {
  console.error("Startup failed", err);
  process.exitCode = 1;
});
