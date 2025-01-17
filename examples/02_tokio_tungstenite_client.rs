use futures_util::{SinkExt as _, StreamExt};
use log::*;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error, Result},
    MaybeTlsStream, WebSocketStream,
};
use url::Url;
use ws_forward::send_slot_subscribe;

#[allow(dead_code)]
const WS_DEVNET: &str = "wss://api.devnet.solana.com";
const WS_DEVELOPMENT: &str = "ws://localhost:8900";

type ChainWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let chain_client = client_to_solana_devnet().await?;

    let (mut write_chain, mut read_chain) = chain_client.split();
    send_slot_subscribe(&mut write_chain).await?;
    while let Some(msg) = read_chain.next().await {
        info!("Chain message: {:?}", msg);
    }

    Ok(())
}

async fn client_to_solana_devnet() -> Result<ChainWebSocket> {
    let (socket, _) =
        connect_async(Url::parse(WS_DEVELOPMENT).expect("Can't connect to case count URL")).await?;
    Ok(socket)
}
