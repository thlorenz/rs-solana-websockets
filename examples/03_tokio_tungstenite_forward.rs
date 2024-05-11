use futures_util::{future, StreamExt, TryStreamExt};
use log::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error, Result},
    MaybeTlsStream, WebSocketStream,
};
use url::Url;

const WS_DEVNET: &str = "wss://api.devnet.solana.com";
const WS_DEVELOPMENT: &str = "ws://localhost:8900";

type ChainWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = "127.0.0.1:9900";

    // -----------------
    // Frontend
    // -----------------

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        let chain_client = client_to_solana_devnet().await?;
        tokio::spawn(accept_connection(chain_client, stream));
    }

    Ok(())
}

async fn accept_connection(chain_socket: ChainWebSocket, stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (write_client, read_client) = ws_stream.split();
    let (write_chain, mut read_chain) = chain_socket.split();
    read_client
        // .try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .inspect(|msg| {
            info!("Client message: {:?}", msg);
        })
        .forward(write_chain)
        .await
        .expect("Failed to forward messages");

    read_chain
        .inspect(|msg| {
            info!("Chain message: {:?}", msg);
        })
        // .try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write_client)
        .await
        .expect("Failed to forward messages");
}

async fn client_to_solana_devnet() -> Result<ChainWebSocket> {
    let (socket, _) =
        connect_async(Url::parse(WS_DEVELOPMENT).expect("Can't connect to case count URL")).await?;
    Ok(socket)
}
