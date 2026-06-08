import { Redis } from "ioredis";

import { REDIS_HOST, REDIS_PORT } from "./redis_constants.js";

export const redis_alert_client = new Redis({
  host: REDIS_HOST,
  port: REDIS_PORT,
});
