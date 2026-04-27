
// You gotta create a function here that will subscribe/ unsuscribe fo a stream by receveing it type and the command

//async fn handle_hl_stream()

// you gonna need to refactor all this mess tmr, because I think messsage is unescssary, since now we have just one channel by separating parts
// by what they should do it should be better allocated right now is a horrible spagheti

use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use crate::market_data::constans::HYPERLIQUID_WS_URL;



pub async fn run_hyperliquid_client(message: String) -> Result<(), Box<dyn std::error::Error>> 
{
    // connect with hype Ws
    let mut ws_stream = connect_ws();

    // send message
    let serialized_msg = serde_json::to_string(&message);

    //ws_stream.send

    Ok(())
}

async fn connect_ws()->Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>
{
    let (ws_stream, response) = 
    connect_async(HYPERLIQUID_WS_URL).
    await.
    expect("Connection Failed");

    println!("Connected with status: {}", response.status());

    Ok(ws_stream)
}

async fn read_message(mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>)
{

    while let Some(result) =  ws_stream.next().await
    {
        match result
        {
            Ok(Message::Text(text)) => 
        }
    }
}

