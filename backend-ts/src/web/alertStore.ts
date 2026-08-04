import { Coin, PriceDirection } from "../redis/subscription_types/common.js";

/* In-memory mirror of the alerts we've asked the engine to track — the engine
itself has no query channel, only one-way subscribe/unsubscribe commands and
one-way fired-alert notifications. We add here when the web UI creates an
alert, and remove here when a fired-alert notification tells us the engine
already removed it (manual price alerts are one-shot). This can drift from
the engine's true state if the backend restarts (state is in-memory only) —
acceptable for a single-user tool, not a source of truth for anything else. */

export type StoredAlert = {
  coin: Coin;
  direction: PriceDirection;
  trigger_price: number;
};

const alerts: StoredAlert[] = [];

function isSameAlert(a: StoredAlert, b: StoredAlert): boolean {
  return a.coin === b.coin && a.direction === b.direction && a.trigger_price === b.trigger_price;
}

export function addAlert(alert: StoredAlert): void {
  if (!alerts.some((existing) => isSameAlert(existing, alert))) {
    alerts.push(alert);
  }
}

export function removeAlert(alert: StoredAlert): void {
  const index = alerts.findIndex((existing) => isSameAlert(existing, alert));
  if (index !== -1) {
    alerts.splice(index, 1);
  }
}

/* All alerts, grouped by coin — every known coin is present, even with an
empty list, so the UI always has a stable set of sections to render. */
export function listAllAlerts(): Record<Coin, StoredAlert[]> {
  const grouped = {} as Record<Coin, StoredAlert[]>;
  for (const coin of Object.values(Coin)) {
    grouped[coin] = alerts.filter((alert) => alert.coin === coin);
  }
  return grouped;
}