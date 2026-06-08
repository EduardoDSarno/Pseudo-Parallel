import { publishSubscribe } from "./redis/publishSubscription.js";
import { redis_alert_client } from "./redis/redis.js";
import { priceSubType } from "./redis/subscription_builders.js";
import { Coin } from "./redis/subscription_types.js";

async function main(): Promise<void> {
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

main()
  .catch((err: unknown) => {
    console.error("Subscription smoke failed", err);
    process.exitCode = 1;
  })
  .finally(() => {
    redis_alert_client.disconnect();
  });
