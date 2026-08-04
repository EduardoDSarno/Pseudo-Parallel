# Liquidation Cluster Tracker

## Goal

Build a small, free tool that records the largest BTC and ETH liquidation
clusters together with the current market price. The first version should focus
on collecting useful observations, not predicting liquidation levels.

## Data Source

HyperPerps exposes a free public API built from observed Hyperliquid positions:

```text
GET https://trade.hyperperps.app/api/public/heatmap/BTC
GET https://trade.hyperperps.app/api/public/heatmap/ETH
```

The API currently requires no account or API key. Its response includes:

- Price when the data was computed.
- Long liquidation clusters below the market.
- Short liquidation clusters above the market.
- Estimated notional value at each level.
- Number of wallets represented by each cluster.
- Percentage distance from the current price.
- Update time, data age, and stale status.

Example cluster:

```json
{
  "price": 61553.01,
  "notional_usd": 98091440,
  "wallet_count": 9,
  "distance_pct": -3.5
}
```

Source documentation:

- [HyperPerps liquidation heatmap](https://hyperperps.app/hyperliquid-liquidation-heatmap)
- [HyperPerps BTC API](https://trade.hyperperps.app/api/public/heatmap/BTC)
- [HyperPerps ETH API](https://trade.hyperperps.app/api/public/heatmap/ETH)

## Important Limitations

HyperPerps is a third-party service, not an official Hyperliquid API. It has no
guaranteed uptime or stable response contract and could introduce limits later.

The data represents observed Hyperliquid positions, not the complete
cross-exchange BTC or ETH market. The provider also excludes cross-margin
positions when a reliable liquidation price cannot be determined.

Every response should therefore be validated before use. Snapshots should be
stored locally so previously collected data remains available if the provider
changes or becomes unavailable.

## V1 Flow

```text
Fetch BTC and ETH every five minutes
                |
                v
Validate the external response
                |
                v
Keep the strongest long and short clusters
                |
                v
Append the complete observation to JSONL
                |
                v
Display price and horizontal cluster lines
```

Each stored snapshot should include:

```text
symbol
observed_at
provider_updated_at
provider_age_seconds
provider_stale
spot_price
sample_size
long_clusters
short_clusters
```

JSONL is suitable for V1 because it is append-only, easy to inspect, and keeps
the historical observations intact. SQLite can replace it later if querying the
history becomes inconvenient.

## Web View

The first chart should show:

- BTC or ETH price.
- Long liquidation levels below the price.
- Short liquidation levels above the price.
- Line thickness based on estimated notional value.
- Cluster price, notional value, wallet count, and distance on hover.
- Last provider update and stale-data warning.

The page should also list the strongest clusters in a table so the information
is still readable without relying only on the chart.

## Expansion Ideas

1. Track when price reaches a previously recorded cluster.
2. Measure how long clusters remain active.
3. Compare cluster strength with the resulting price movement.
4. Add notifications when a large cluster appears near the market price.
5. Move snapshots from JSONL to SQLite.
6. Compare Hyperliquid clusters with completed liquidation data from the free
   [Coinalyze API](https://api.coinalyze.net/v1/doc/).

## Other Sources Considered

- CoinGlass provides free BTC and ETH heatmaps on its website, but its
  liquidation-map API requires a paid professional plan.
- CoinAnk advertises a seven-day trial, but ongoing liquidation-map API access
  is paid.
- Coinalyze has a free API for completed liquidation history, but it does not
  provide future liquidation price clusters.
