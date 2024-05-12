use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{Error, Result};
use ws_forward::{client_to_solana_development, client_to_solana_devnet, ChainWebSocket};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = "127.0.0.1:9900";

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        let devnet_client = client_to_solana_devnet().await?;
        let local_client = client_to_solana_development().await?;
        tokio::spawn(accept_connection(devnet_client, local_client, stream));
    }

    Ok(())
}

async fn accept_connection(
    chain_socket: ChainWebSocket,
    ephem_socket: ChainWebSocket,
    stream: TcpStream,
) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (mut write_client, mut read_client) = ws_stream.split();
    let (mut write_chain, mut read_chain) = chain_socket.split();
    let (mut write_ephem, mut read_ephem) = ephem_socket.split();
    loop {
        tokio::select! {
            // We pipe both chain and ephemeral messages to the client
            next = read_chain.next() => {
                match next {
                    Some(msg) => {
                        debug!("Chain message: {:?}", msg);
                        write_client.send(msg.unwrap()).await.unwrap();
                    }
                    None => {
                        info!("Chain stream ended");
                        break;
                    }
                }
            }
            next = read_ephem.next() => {
                match next {
                    Some(msg) => {
                        debug!("Ephem message: {:?}", msg);
                        write_client.send(msg.unwrap()).await.unwrap();
                    }
                    None => {
                        info!("Ephem stream ended");
                        break;
                    }
                }
            }
            // For client messages we decide by message content if to send it
            // to chain or ephem socket
            next = read_client.next() => {
                match next {
                    Some(msg) => {
                        debug!("Client message: {:?}", msg);
                        // Obviously the decision making would be totally different, but
                        // for our case we can use the `id = "ephem"` to force it to go
                        // to the ephemeral RPC.
                        if msg.as_ref().unwrap().to_text().unwrap().contains("ephem") {
                            write_ephem.send(msg.unwrap()).await.unwrap();
                        } else {
                            write_chain.send(msg.unwrap()).await.unwrap();
                        }
                    }
                    None => {
                        info!("Client stream ended");
                        break;
                    }
                }
            }
        };
    }
}
