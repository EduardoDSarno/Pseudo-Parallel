# TODO

## Current application

The Rust engine subscribes to the Hyperliquid HYPE M5 candle stream, keeps the
latest and recently closed candles in memory, evaluates one-shot manual price
crossings, and publishes fired alerts to the TypeScript backend through Redis.

Current flow:

```text
Redis subscription -> PriceAlertService
Hyperliquid M5 update -> CandleStore -> take_crossed -> Redis fired alert
```

## Next work by priority

1. Decide the exact spike definition before adding spike detection. The current
   candle store keeps closed M5 candles and the latest forming candle, but does
   not store every WebSocket price update.
2. Add liquidation ingestion only after selecting the API and defining the
   internal liquidation model.

## Deferred

- User alert catalog (mapping a triggered price level back to every subscribed
  user). This is a single-user project right now — there's no user concept in
  the system at all yet. Revisit once/if multi-user support is an actual
  requirement, not before.

## Removed intentionally

- ATR and generic indicator rules/evaluators.
- REST candle warmup and strict startup seeding.
- M1, M15, and H1 candle streams.
- `SignalInput`, `Event`, `EventEvaluator`, and `PriceEvaluator` wrappers.
- Duplicate coin-level last-price state outside `CandleStore`.
