use futures_util::SinkExt as _;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Result;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;
use url::Url;

use futures_util::stream::SplitSink;

pub type ChainWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn send_slot_subscribe(
    write_chain: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
) -> Result<(), tungstenite::Error> {
    write_chain
        .send(Message::Text(
            "{\n  \"jsonrpc\": \"2.0\",\n  \"id\": 1,\n  \"method\": \"slotSubscribe\"\n}"
                .to_string(),
        ))
        .await
}

const WS_DEVNET: &str = "wss://api.devnet.solana.com";
const WS_DEVELOPMENT: &str = "ws://localhost:8900";

pub async fn client_to_solana_devnet() -> Result<ChainWebSocket> {
    let (socket, _) =
        connect_async(Url::parse(WS_DEVNET).expect("Can't connect to case count URL")).await?;
    Ok(socket)
}

pub async fn client_to_solana_development() -> Result<ChainWebSocket> {
    let (socket, _) =
        connect_async(Url::parse(WS_DEVELOPMENT).expect("Can't connect to case count URL")).await?;
    Ok(socket)
}
