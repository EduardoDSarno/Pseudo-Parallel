# Pseudo Parallel - v1.0 Design

## Problem
The problem I want to solve is literally not having to stare at the screen/chart if I have a position open, so that I could relax and not be scared all the time of some drastic shift in the market made by news or market events that affect the asset.

## Goals (v1)
- The user is alerted when HYPE undergoes a volatility breakout on any of three timeframes (5m, 15m, 1h).
- The system recovers from process restarts automatically, without user intervention or reconfiguration.
- User-configured alert rules survive process restarts and server redeploys.
- The system should have user-configured price levels that will trigger messages on Telegram when price is hit.
- The system will send (if selected so) push alerts of cascade liquidations.
- The system should have a simple interface on which these settings can be configured.
- Alerts are unique and not duplicated.

## Non-Goals (v1)
- I am not going to implement charts because the goal of the app is to be away from the screen.
- I will not add any AI because this is an area that I haven't worked with implementation yet, and it would completely stop me; also, the cost.
- I will not add so many complex alert configurations at first, because I wanna first test how the alerts will react daily, and if I will actually find it useful.
- I'm just gonna use Telegram as notification push because it is the one I am most familiar with and there is no overhead like adding a phone number or email.
- I will just start by following one token (HYPE), but I will leave space for other tokens to be added later on, because I wanna keep it simple to prove an idea before testing.
- I am not gonna add the news and sentiment part because it's just too much work for a v1 and I don't have formulated exactly how to do it.

## Users
Currently it will be just me, so I won't build a public sign-up page, etc. But authentication exists only to keep the Telegram chat ID and alert rules tied to the deployed instance, not to support multiple accounts.

## Architecture Overview
My system will have 7 main parts.

### Web Frontend
We will use TypeScript with React, for the alert setup page and view alerts.

### A Web Backend
This will be a Node/TS controller that will trigger notifications and actions from the UI, database (ex: write the rules), and push rule changes to engine.

### Data Engine
This will be the engine in Rust that will run on its own, grabbing and working with the data all the time; the backend fires notifications when some rule changes.

### Message Queue
I will be using Redis. The queue will be a method of communication between backend and engine without directly connecting each other. The backend will push messages there, the engine will be subscribed to it, and it will read the payload directly from there or it will query the database (still just a signal).

### Database
I will be using Postgres, holding users, Telegram chat IDs, alert rules, and alert history.

### Push Notifications
The push notifications will be made by Telegram APIs, which will be triggered automatically by the data engine that will be running 24/7. The data engine also must store data of the alert in DB, so the alert information is saved for future reference.

### Data Source
The current program data source will be Hyperliquid API; it is where the engine will run based on. We will establish a WebSocket stream connection with Hyperliquid, and the engine will fetch the data and process accordingly.

## Data Flow
The platform overview will be basically starting from the UI frontend, where the user will set its first alerts and confirm. With that, our backend will trigger an action to send information to be stored in the database (such as alert details). That would also send a message to the queue where the data engine would receive a message, and it would query the DB for current rule set. Then the engine evaluates rule with its data received from Hyperliquid continuously until it's hit and sends the notification to the Telegram account linked to the rule.

Important: On restart, the engine will query the DB for all enabled rules to rebuild its memory.

### Data Model
- Users (email, password hash, created_at)
- Telegram chat IDs (linked to user)
- Alert rules (user_id, asset, rule_type, parameters, enabled)
- Alert history log (user_id, rule_id, fired_at, payload, delivery_status)

- Data ingestion will run as a persistent server-side process, not dependent on browser sessions.
- Persist data that is authoritative (originates in your system). Don't persist data that is derived (can be re-fetched from a source of truth).

### Persistent
They are divided into:

1. Persist (database): user accounts, Telegram chat IDs, user-configured price level alerts, user-configured ATR thresholds, historical alert log (so you can see "what did it fire last week"), dedup state (maybe - we'll get to this).

2. Do NOT persist: candles, live price, liquidation events, WebSocket state, ATR values. Re-fetch/recompute on startup.

## Open Questions (for future)
- Where am I going to host?
- Do I still add login/signup even if it is just for me?
- Am I going to resend all alerts that were not sent on restart? (liquidation events, notifications)
- Am I going to encrypt private information?

## Key Decisions
For this platform I have made some design decisions that reflect the goal of the project, learning.

- Message Queue: I chose Redis, because it is the most used library for this purpose and has a very rich environment, and I believe it will be a great thing to learn.
- Database: I chose Postgres because I have used it before (even though, I don't have familiarity with it) and because of its push message system that I believe can be useful in the future.
- For this project I decided to use Rust as a data engine because of its speed and safety for developing. Because I am a new developer, my chances of committing a memory mistake or bug are considerably high; Rust will help avoid that. Besides, it is a language which I know a bit and I believe is very valuable to learn in the future.
- For web backend and frontend my choice was TypeScript for a couple of reasons. The first is that TypeScript is currently the most used language on GitHub, making it really valuable to know. Second, the world of JS/TS for frontend is the go-to option for its frameworks, such as React, which I will be using. Their handling of backend as well with Node.js is very useful and a good bridge between a lower-level language (Rust) and the frontend.
