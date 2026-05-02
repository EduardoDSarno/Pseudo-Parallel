
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use crate::market_data::{constans::HYPERLIQUID_WS_URL, hyperliquid::protocols::inbound::InboundMessage, types::candle::Candle};



pub async fn run_hyperliquid_client(message: String) -> Result<(), Box<dyn std::error::Error>> 
{
    // connect with hype Ws
    let mut ws_stream = connect_ws_hl().await?;

    // send message 
    ws_stream.send(Message::Text(message)).await?;


    while let Some(result) =  ws_stream.next().await
    {
        // It will return false when closed
        if !read_message(result)
        {
            break;
        }
    }

    Ok(())
}

/* This function it returns a websocketsream conneciton with hyperlquid */
async fn connect_ws_hl() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>
{
    let (ws_stream, response) = 
    connect_async(HYPERLIQUID_WS_URL).
    await.
    expect("Connection Failed");

    println!("Connected with status: {}", response.status());

    Ok(ws_stream)
}

/* THis function will read the message and try to match (if successfully received text)
    with one of our message inbounds otherwise it will match with different types of responses
     */
fn read_message(result: Result<Message, tokio_tungstenite::tungstenite::Error>) -> bool
{
    match result
    {
        Ok(Message::Text(text)) => 
        {
            let deserialized = serde_json::from_str::<InboundMessage>(&text);
            let _= match_response(deserialized);
            true
        }
        // Tokio tungstain handles automatically
        Ok(Message::Ping(_message)) =>
        {
            println!("Received ping");
            true
        }
        Ok(Message::Pong(_message))=>
        {
            println!("Received Pong");
            true
        }
        Ok(Message::Close(close_frame)) => 
        {
            println!("WebSocket closed: {:?}", close_frame);
            false
        }
        _ => 
        {
            println!("Received an unexpected WebSocket message type or encountered an error in the stream.");
            true
        },

    }
    
}

/* THis function is soly responsible for matching the message with one of our inbounds streams */
fn match_response(message_response: Result<InboundMessage, serde_json::Error>) -> Result<(), Box<dyn std::error::Error>>
{

    match message_response
    {
        Ok(InboundMessage::SubscriptionResponse(response))=>
        {
            println!("{:?} Successeful. Steam: {:?}",response.method, response.subscription);
            Ok(())
        }

        Ok(InboundMessage::Candle(candle_hl)) => 
        {
            let candle = Candle::try_from(candle_hl)?;
            println!("{:#?}", candle);
            Ok(())
        }
        Err(err) => {
            // failed to parse JSON
            println!("Failed to parse JSON, message: {:#}", err);
            Err(Box::new(err))
       
        }
    }
}
