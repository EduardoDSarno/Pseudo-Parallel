# Pseudo Parallel - v1.0 Design

## Problem
The problem I want to solve is literally don't have to stare at the screen/chart if I have a position open, that I could relax and don't be scared all the time of some drastic shift in the market made by news or market events that affect the asset

## Goals (v1)
-The user is alerted when HYPE undergoes a volatility breakout on any of three timeframes (5m, 15m, 1h).
-The system recovers from process restarts automatically, without user intervention or reconfiguration.
- User-configured alert rules survive process restarts and server redeploys
- The system should have user configured price levels that will trigger message on telegram when price is hit
- The system will send (if selected so) push alerts of cascade liquidations
- The system should have a simple interface on which these settings can be configured
- Alerts are unique and not duplicated

## Non-Goals(v1)
- I am not going to implement charts because the goal of the app is to be away from 
 the screen
 - I will not add any AI because is a area that I have't worked with implementation yet,
 and it would completely stop me, also the cost
 - I will not add so many complex alert configurations at first, because I wanna first test how the alerts will react daily, and if I will actually find it usefull
 - I'm just gonna use telegram as notification push because is the one I am most familiar with and there is no overhead like adding a phone number or email
- I will just start by following one token (hype) but I will leave space for other tokens be added later on, because I wanna keep it simple to prove and idea before testing
- I am not gonna add the news and sentiment part because it's just to much work for a v1 and I don't have formulated exaclty how to do it.

## Users 
Currenlty will be just me so I won't build a public sign up/ page etc. But Authentication exists only to keep the Telegram chat ID and alert rules tied to the deployed instance, not to support multiple accounts.

## Architecture Overview
My system will have 3 main parts

### Web Frontend 
We will use typescript with react, for the alert set up page and view alerts

### A Web backend
This will be a Node/ts controller that will trigger notification and actions from the UI, database, and push rules CHANGES to engine

### Data engine
This will be the engine in rust where we will fetch the data and trigger alerts based on API calls from the backend


# Data Flow


## Persistent Data Structure (Database)
-Users (email, password hash, created_at)
-Telegram chat IDs (linked to user)
-Alert rules (user_id, asset, rule_type, parameters, enabled)
-Alert history log (user_id, rule_id, fired_at, payload, delivery_status)

-Data ingestion will run as a persistent server side process, not dependent of browser sessions

- Persist data that is authoritative (originates in your system). Don't persist data that is derived (can be re-fetched from a source of truth).

They are divided into :

1: Persist (database): user accounts, Telegram chat IDs, user-configured price level alerts, user-configured ATR thresholds, historical alert log (so you can see "what did it fire last week"), dedup state (maybe — we'll get to this).

2:
Do NOT persist: candles, live price, liquidation events, WebSocket state, ATR values. Re-fetch/recompute on startup. 