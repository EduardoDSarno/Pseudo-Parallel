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

## Next work

1. Add the user alert catalog so one triggered price level can be connected back
   to every user subscribed to it.
2. Validate incoming fired-alert JSON in the TypeScript backend instead of
   trusting the result of `JSON.parse`.
3. Decide the exact spike definition before adding spike detection. The current
   candle store keeps closed M5 candles and the latest forming candle, but does
   not store every WebSocket price update.
4. Add liquidation ingestion only after selecting the API and defining the
   internal liquidation model.

## Operational follow-ups

- Replace the unbounded alert publisher channel with a bounded channel and an
  explicit overflow policy.
- Decide whether Redis Pub/Sub delivery is sufficient or fired alerts need a
  durable Redis Stream.
- Add an integration test that runs Redis, publishes a price subscription, and
  verifies the fired alert received by the TypeScript subscriber.

## Removed intentionally

- ATR and generic indicator rules/evaluators.
- REST candle warmup and strict startup seeding.
- M1, M15, and H1 candle streams.
- `SignalInput`, `Event`, `EventEvaluator`, and `PriceEvaluator` wrappers.
- Duplicate coin-level last-price state outside `CandleStore`.
