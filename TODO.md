# TODO

## Finish `market_data` (before leaving the module)

Complete the **medium** list below before alert subscription stream / API / `backend-ts` wiring. Do **evaluator pass last** — live spot-check is the gate for leaving the module.

### High (correctness / ops) — done

- [x] **`market_view` missing** — `signals.rs` logs why (buffer / `last_seen` / `closed_len`); `debug` when warming up, `warn` when unexpected.
- [x] **Seed strictness** — 3 REST attempts with backoff, fail if still short; `last_seen` on seed; `verify_seeded_keys`; ingest dedup on bar roll.

### Medium (clarity / maintainability)

Do in order; **evaluator pass** is the last item before the module is finished.

- [x] **Name/document candle store role** — `engine` module renamed to `candle_store`; `Engine` → `CandleStore`, `MarketView` → `CandleView`, `runtime.candle_store`.
- [ ] **Fill `engine-rust/docs/ENGINE.md`** — candle store contract, REST seed, `CandleView` requirements (rename doc from “engine” to candle store).
- [x] **Optional integration test** — `signals_tests.rs`: M5 price path, indicator rules → `AtrBreakout` through `run_signals` (no HTTP).
- [x] **Per-user indicator thresholds (infrastructure)** — `IndicatorRuleService`, `IndicatorRule` / `AtrRule`, config defaults, `arm_default_indicator_rules`; ATR eval uses rule thresholds; dedup by `IndicatorRuleId`. **Still later:** user id, catalog, subscribe/unsubscribe API.
- [ ] **Evaluator pass** *(last — finish `market_data`)* — unit tests in place; remaining live spot-check:
  - Manual price: directional cross on M5 **close** only (`PRICE_ALERT_INTERVAL_MS`).
  - ATR (v1): baseline + signal TR from **closed** bars only; eval when `bar_just_closed`; spike-level dedup per `IndicatorRuleId`.
  - Flow: `process` → `candle_ingest` → `run_signals` → `dispatch`.
  - Run `cargo test`; live grep for `Manual price alert triggered` / `ATR breakout detected`.

### Candle store vs orchestrator (v1)

- `MarketUpdate` / `SignalInput` = extensible ingress and signals.
- `CandleStore` = candle-only storage (`CandleKey` buffers + `last_seen`) — intentional until a second data type exists.
- `MarketDataRuntime` = composition root (candle store, price book, indicator rules, evaluators).

---

## Next work (after `market_data` is done)

### 1. Alert subscription stream

- Wire UI/API → `MarketDataRuntime::alert_service_mut().subscribe(...)`.
- Wire indicator rules → `indicator_rule_service_mut().subscribe(...)` (when API exists).
- **Auto direction on subscribe** — user supplies coin + trigger price only; set `ManualPriceDirection` from last coin close: if `current < trigger` → **Above**, if `current > trigger` → **Below** (so we always wait for the obvious cross). Reject or special-case `current == trigger`.
- On user delete/disarm → `unsubscribe` (decrement `subscriber_count`; remove level when 0).
- Log subscribe/unsubscribe with user id when available (coin, direction, trigger price).
- Lives **outside** the candle pipeline (API / second task in `main` / `backend-ts`), calling into runtime.

### 2. User catalog (multi-user, later)

- Per-user alert rows: definition + `armed` toggle (catalog row stays when book level disarms on trigger).
- On trigger: notify all armed users on that level; set `armed = false` in catalog.
- Fan-out in `dispatch` when catalog exists (use `indicator_rule_id` on `AtrBreakout`).

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
- M5-only price gate (`PRICE_ALERT_INTERVAL_MS`) — M15/H1 updates do not move `last_market_price`.
- Range crossing: above / below with `PriceKey` in `BTreeMap`.
- **Trigger once:** on fire → `disarm_levels` / `delete_level` (whole slot removed; re-arm via subscribe/toggle later).
- `unsubscribe` = decrement subscriber; trigger = full level delete.

### Indicator rules (ATR)

- `IndicatorRuleService` + `IndicatorRule` / `IndicatorRuleKind` / `AtrRule`.
- Default ATR rules armed per `CandleKey` after REST seed (`arm_default_indicator_rules`).
- `AtrEvaluator`: thresholds from rule; dedup state keyed by `IndicatorRuleId`.
- `Event::AtrBreakout` includes `indicator_rule_id`.

### Runtime & orchestrator

- `MarketDataRuntime` = composition root (`candle_store`, `PriceAlertService`, `IndicatorRuleService`, `EventEvaluator`).
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

- `EventEvaluator`: `evaluate_price` + `evaluate_indicators(rules)` (no owned alert storage).
- `CandleView` for indicator path; price path uses M5 close + book only.

### Hyperliquid client

- `hl_client`: reconnect forever with backoff; resubscribe from `&[CandleKey]` each session.
- `stream_health`: per-key staleness warn (~2× interval); health tick in `select!`.
- Constants in `constans.rs` (reconnect + stream health); tests in `stream_health_tests.rs`.
- `Hyperliquid.md` updated (reconnect + stream health).

### Tests

- Price book + service + stream health + seed/ingest + indicator rules + ATR evaluator + signals pipeline (`cargo test` — 45+).

---

## Design choices (current)

| Topic | Decision |
|--------|-----------|
| Book vs catalog | **Book** = active armed levels for crossing. **Catalog** (future) = all user alerts + `armed` flag. |
| Trigger once (price) | Book level removed on cross; user row stays disarmed until re-subscribe. |
| ATR alerts | Re-fire on higher spike level same bar; dedup per `IndicatorRuleId`, not one-shot disarm. |
| Shared level | `subscriber_count` on `PriceLevelEntry`; one cross notifies all (catalog fan-out later). |
| Price source | Live M5 candle **close** for manual price (`PRICE_ALERT_INTERVAL_MS`). |
| Indicator rules | Per `CandleKey`; thresholds in `AtrRule`; defaults from `MarketDataConfig`. |
| Logging | Orchestrator = pipeline decisions; `dispatch` = per-alert payload. |
| Evaluators | Stateless price eval; ATR state in `AtrEvaluator` keyed by rule id. |

---

## Out of date — do not follow

- ~~`AlertBook`~~ → use **`PriceAlertService`** / **`PriceBook`**.
- ~~`Engine` / `MarketView`~~ → use **`CandleStore`** / **`CandleView`**.
- ~~Evaluator should not delete alerts~~ → price book **disarms on trigger**; catalog disarm is separate (future).
- ~~Coordinator `handle_candle` does everything~~ → use **`process`** + modules above.
- ~~Decide one-shot vs recurring~~ → **one-shot at book level** for price; ATR uses spike-level ladder.
- ~~Global `ATR_BREAKOUT_RATIO` only in evaluator~~ → rule thresholds via **`IndicatorRuleService`**.
