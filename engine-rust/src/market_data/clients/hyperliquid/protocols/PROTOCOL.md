## Hyperliquid Protocol

The current protocol for Hyperliquid communications was made to simplify/modularize and facilitate modifications that include addition/removal of streams into our program.

## Connection Flow

Starting from inner, our Hyperliquid has a WebSocket way of connection, which consists of 4 steps:

### 1. WebSocket Connection

1. Is connection to the secure WebSocket (source link in constants.rs).

### 2. Stream Request

2. The first consists of sending a request for connection to a stream, which is formatted as:

```json
{ "method": "subscribe", "subscription": { "type": "trades", "coin": "SOL" } }
```

Which has a couple variants:

- Its methods include subscribe, as you see above, and unsubscribe, which we resolved by creating an enum for it.
- The subscriptions are trickier because they englobe (for what I've seen) a couple of combinations that are based on type. To deal with this, we created an enum on which we selected just a few to include and we will need to add later on if desired.

### 3. Subscription Ack

3. The third step is dealing with the Ack from the WebSocket, confirming a successful connection to x channel, ex:

```json
{"channel":"subscriptionResponse","data":{"method":"subscribe","subscription":{"type":"trades","coin":"SOL"}}}
```

The handling of this structure is made inside inbound.rs, which is a wrapper into the subscription data found inside subscribe.rs.

### 4. Channel Data

4. The last step is after receiving Ack, we will start getting the data from the type of channel we subscribed to, and for each channel a new type would have to be added (data_models). Since we are currently handling only candles, we only add that one, and a match statement is used to match and then construct these structures.