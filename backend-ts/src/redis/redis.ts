import { Redis } from "ioredis";

import { REDIS_HOST, REDIS_PORT } from "./redis_constants.js";

// Client for publishing alerts to the alert channel
export const redis_alert_client = new Redis({
  host: REDIS_HOST,
  port: REDIS_PORT,
});

// Client for subscribing to the alert channel
export const redis_alert_subscriber = new Redis({
  host: REDIS_HOST,
  port: REDIS_PORT,
});
