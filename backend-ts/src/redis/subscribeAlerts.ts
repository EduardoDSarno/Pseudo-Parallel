import {redis_alert_subscriber} from "./redis.js";
import { ALERTS_FIRED_CHANNEL } from "./redis_constants.js";
import { isOutgoingAlert, type OutgoingAlert } from "./subscription_types/outgoing.js";

export async function subscribeAlerts(callback: (alert: OutgoingAlert) => void)
{
    const count = await redis_alert_subscriber.subscribe(ALERTS_FIRED_CHANNEL);
    console.log(`Subscribed to ${ALERTS_FIRED_CHANNEL} channel. Count: ${count}`);

    redis_alert_subscriber.on("message", (channel, message) =>
    {
        if (channel === ALERTS_FIRED_CHANNEL)
        {
            let parsed: unknown;
            try {
                parsed = JSON.parse(message);
            } catch (err) {
                console.error("Received malformed alert JSON, skipping:", err, message);
                return;
            }

            if (!isOutgoingAlert(parsed)) {
                console.error("Received alert with unexpected shape, skipping:", parsed);
                return;
            }

            callback(parsed);
        }
    });
}

