## Hyperliquid Market Data Module

The Hyperliquid Market Data Module is responsible for being an adapter between the rest of our application and the
Hyperliquid structure of capturing data.

It is broken down in parts with protocols that will contain the protocol we will follow to capture data from the Hyperliquid
WebSocket connection stream. As of May 13, 2026, we just have inbound and subscribe, which are responsible for handling
the messages we send to Hyperliquid and the messages we receive back from it.

The goal of this module is to hide the Hyperliquid-specific format from the rest of our code. The rest of the application
should not need to know that Hyperliquid uses fields like `t`, `T`, `s`, `i`, or that prices arrive as strings. This module
receives that external structure and turns it into the internal structures that our engine can use.

## Current Structure

The current Hyperliquid module is organized like this:

```text
hyperliquid/
    hl_client.rs
    protocols/
        subscribe.rs
        inbound.rs
        data_models/
            candle.rs
```

`hl_client.rs` is responsible for the WebSocket connection itself. It connects to Hyperliquid, sends the subscription
requests, keeps listening to the stream, and sends the valid market data forward to the rest of the program.

`protocols/subscribe.rs` is responsible for the outbound messages, meaning the messages that we send to Hyperliquid.
This is where we define the subscribe/unsubscribe method and the different subscription types, such as candle, l2 book,
trades, and user events.

`protocols/inbound.rs` is responsible for the inbound messages, meaning the messages Hyperliquid sends back to us. This
includes subscription responses, candle data, and error messages.

`protocols/data_models/candle.rs` is responsible for the raw Hyperliquid candle structure. This is not our final internal
candle structure. It is just the shape of the data exactly as Hyperliquid sends it.

## WebSocket Flow

The main flow starts from the caller, where we create the subscriptions we want and pass them into `run_hyperliquid_client`.

The Hyperliquid client then follows this order:

1. Connect to the Hyperliquid WebSocket URL.
2. Serialize each subscription into JSON.
3. Send each subscription through the WebSocket.
4. Keep listening for incoming WebSocket messages.
5. If the message is text, try to deserialize it into `InboundMessage`.
6. Match the inbound message type.
7. If the message is a candle, convert it from `CandleHL` into our internal `Candle`.
8. Send the internal `Candle` into the event/engine flow.

So the simplified flow looks like this:

```text
main.rs
    -> run_hyperliquid_client
        -> connect_ws_hl
        -> send subscriptions
        -> read_message
        -> match_response
        -> CandleHL
        -> Candle
        -> handle_candle_event
```

## Outbound Messages

Outbound messages are messages we send to Hyperliquid.

For example, a candle subscription is expected to look like this:

```json
{
  "method": "subscribe",
  "subscription": {
    "type": "candle",
    "coin": "HYPE",
    "interval": "5m"
  }
}
```

In code, this is represented by `SubscribeToChannelReq`, `Method`, and `SubscriptionData`.

`Method` represents what we want to do:

- `SUBSCRIBE`
- `UNSUBSCRIBE`

`SubscriptionData` represents what channel we want:

- `Candle { candle_key }`
- `L2Book { coin }`
- `Trades { coin }`
- `UserEvents { user }`

Right now the helper we use most is `subscribe_candle`, because the current focus of the engine is candle data.

## Inbound Messages

Inbound messages are messages Hyperliquid sends back to us.

The response format is wrapped by a `channel` field and a `data` field. For example:

```json
{
  "channel": "subscriptionResponse",
  "data": {
    "method": "subscribe",
    "subscription": {
      "type": "candle",
      "coin": "HYPE",
      "interval": "5m"
    }
  }
}
```

This is why `InboundMessage` uses serde with `channel` as the tag and `data` as the content. The enum lets us match what
kind of inbound message we got without manually checking strings everywhere.

The current inbound variants are:

- `SubscriptionResponse`
- `Candle`
- `Error`

## Candle Data

Hyperliquid candle data comes in a compact format, using short field names:

- `t` means open time in milliseconds.
- `T` means close time in milliseconds.
- `s` means coin.
- `i` means interval.
- `o` means open price.
- `c` means close price.
- `h` means high price.
- `l` means low price.
- `v` means volume.
- `n` means number of trades.

We model this as `CandleHL`, because this is the Hyperliquid version of a candle.

The important detail is that prices and volume arrive as strings, not as numbers. Because of that, we do not use `CandleHL`
directly inside the engine. We first convert it into our internal `Candle`, where prices are numeric values and can be used
for calculations.

## Current Responsibility Boundary

The Hyperliquid module should care about:

- How to connect to Hyperliquid.
- How to subscribe to Hyperliquid streams.
- How to deserialize Hyperliquid responses.
- How to convert Hyperliquid candle data into our internal candle type.

The Hyperliquid module should not care too much about:

- How the engine stores candles.
- How indicators are calculated.
- How breakout signals are evaluated.

Right now `hl_client.rs` still receives a mutable `Engine`, so the client is a little coupled to the engine. That is okay
for now because the project is still small, but later if we add another market data source, we may want the client to return
events instead of calling the engine directly.

## Future Additions

To add a new Hyperliquid stream, the flow will probably be:

1. Add the subscription variant inside `SubscriptionData`.
2. Add the inbound variant inside `InboundMessage`.
3. Add the raw data model inside `data_models`.
4. Add the conversion into our internal type.
5. Add the event handling that sends it into the correct part of the engine.

For now, candles are the only complete flow from Hyperliquid all the way into the engine.