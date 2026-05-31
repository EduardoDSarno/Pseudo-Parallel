# TODO

## Finish `market_data` (before leaving the module)

Complete this list before alert subscription stream / API / `backend-ts` wiring.

### High (correctness / ops)

- [x] **`market_view` missing** — `signals.rs` logs why (buffer / `last_seen` / `closed_len`); `debug` when warming up, `warn` when unexpected.
- [x] **Seed strictness** — 3 REST attempts with backoff, fail if still short; `last_seen` on seed; `verify_seeded_keys`; ingest dedup on bar roll.
- [ ] **Evaluator pass** — confirm cross rules + ATR behavior:
  - Manual price: directional cross on coin **close** only (not wick-only / high-low containment).
  - ATR: baseline from closed buffer; live TR from current candle; spike-level dedup in ATR evaluator.
  - Flow: `process` → `candle_ingest` → `run_signals` → `dispatch`.

### Medium (clarity / maintainability)

- [ ] **Name/document engine role** — comment or `CandleEngine` alias so it’s clear `Engine` is candle storage, not whole runtime state.
- [ ] **Fill `engine-rust/docs/ENGINE.md`** — candle store contract, REST seed, `MarketView` requirements.
- [ ] **Optional integration test** — ingest → signals → disarm (no HTTP).

### Engine vs orchestrator (v1 — no generic engine yet)

- `MarketUpdate` / `SignalInput` = extensible ingress and signals.
- `Engine` = candle-only store (`CandleKey` buffers + `last_seen`) — intentional until a second data type exists.

---

## Next work (after `market_data` is done)

### 1. Alert subscription stream

- Wire UI/API → `MarketDataRuntime::alert_service_mut().subscribe(...)`.
- On user delete/disarm → `unsubscribe` (decrement `subscriber_count`; remove level when 0).
- Log subscribe/unsubscribe with user id when available (coin, direction, trigger price).
- Lives **outside** the candle pipeline (API / second task in `main` / `backend-ts`), calling into runtime.

### 2. User catalog (multi-user, later)

- Per-user alert rows: definition + `armed` toggle (catalog row stays when book level disarms on trigger).
- On trigger: notify all armed users on that level; set `armed = false` in catalog.
- Fan-out in `dispatch` when catalog exists.

### 3. Dispatch beyond logging

- `coordinator/dispatch.rs` today only `tracing::info!`.
- Later: WebSocket, UI, push.

### 4. Future market inputs

- Add `MarketUpdate` + `SignalInput` variants (order book, trades, etc.).
- Per-type ingest module; build `SignalInput` from ingest result (not only `IngestedCandleSnapshot`).

---

## Completed (architecture & bugs)

### Price alerts

- `PriceBook` + `PriceAlertService` + stateless `PriceEvaluator`.
- Coin-level price gate (`last_market_price_by_coin`) — no per-interval fake crosses.
- Range crossing: above / below with `PriceKey` in `BTreeMap`.
- **Trigger once:** on fire → `disarm_levels` / `delete_level` (whole slot removed; re-arm via subscribe/toggle later).
- `unsubscribe` = decrement subscriber; trigger = full level delete.

### Runtime & orchestrator

- `MarketDataRuntime` = composition root (engine, `PriceAlertService`, `EventEvaluator`).
- Pipeline:
  - `market_update.rs` — `MarketUpdate`
  - `candle_ingest.rs` — `apply_candle` → `IngestedCandleSnapshot`
  - `orchestrator.rs` — `process` (pipeline logs)
  - `signal_input.rs` — `SignalInput`
  - `signals.rs` — `run_signals` (price + indicators, returns `Vec<Alert>`)
  - `alerts.rs` — price eval + disarm
  - `dispatch.rs` — per-alert detail logs
  - `indicators.rs` — indicator eval wrapper
- Entry: `runtime.process(MarketUpdate::Candle(c))` from `hl_client`.

### Signals

- `EventEvaluator`: `evaluate_price` + `evaluate_indicators` (no owned alert storage).
- `MarketView` for indicator path; price path uses coin close + book only.

### Hyperliquid client

- `hl_client`: reconnect forever with backoff; resubscribe from `&[CandleKey]` each session.
- `stream_health`: per-key staleness warn (~2× interval); health tick in `select!`.
- Constants in `constans.rs` (reconnect + stream health); tests in `stream_health_tests.rs`.
- `Hyperliquid.md` updated (reconnect + stream health).

### Tests

- Price book + service + stream health + seed/ingest unit tests (28 total in `engine-rust`).

---

## Design choices (current)

| Topic | Decision |
|--------|-----------|
| Book vs catalog | **Book** = active armed levels for crossing. **Catalog** (future) = all user alerts + `armed` flag. |
| Trigger once | Book level removed on cross; user row stays disarmed until re-subscribe. |
| Shared level | `subscriber_count` on `PriceLevelEntry`; one cross notifies all (catalog fan-out later). |
| Price source | Live candle **close** at coin level (any interval update can move coin price). |
| Logging | Orchestrator = pipeline decisions; `dispatch` = per-alert payload. |
| Evaluators | Stateless price eval; ATR state in indicator/ATR path. |

---

## Out of date — do not follow

- ~~`AlertBook`~~ → use **`PriceAlertService`** / **`PriceBook`**.
- ~~Evaluator should not delete alerts~~ → book **disarms on trigger**; catalog disarm is separate (future).
- ~~Coordinator `handle_candle` does everything~~ → use **`process`** + modules above.
- ~~Decide one-shot vs recurring~~ → **one-shot at book level** for now; user toggle = re-arm.
