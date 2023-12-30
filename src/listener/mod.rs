use crate::config::Network;
use bitcoin::message_type::MessageType;
use bitcoin::verack::VerAck;
use bitcoin::version::{VersionBuilder, VersionBuilderError};
use bitcoin::{Message, Payload, SerdeBitcoin, SerdeBitcoinError};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::{error, info};

#[derive(Default, Debug, Clone)]
pub enum ConnectionStatus {
    #[default]
    NoConnection,
    Connecting,
    Connected,
}

// @TODO: It doesn't need the DashMap, but if the state were to be shared among the tasks, then it would come quite handy
pub async fn run(
    mut stream: TcpStream,
    network: Arc<Network>,
    connections: Arc<DashMap<SocketAddr, ConnectionStatus>>,
) -> Result<(), Error> {
    let testnet = network.is_testnet();
    let addr = stream.peer_addr().map_err(Error::FailedToGetPeerAddr)?;

    loop {
        let status = connections
            .get(&addr)
            .map(|v| v.value().clone())
            .unwrap_or_default();
        // Read the message
        let mut br = BufReader::new(&mut stream);
        let mut response_buffer = br.fill_buf().await.map_err(Error::FillBuffer)?.to_vec();
        stream.flush().await.map_err(Error::FailedToFlushStream)?;

        if response_buffer.is_empty() {
            // Connection closed by the peer
            return Ok(());
        }

        // Deserialize the response
        let message: Message = Message::deserialize(&mut response_buffer)
            .map_err(Error::DeserializeVersionResponse)?;

        let new_status = match status {
            ConnectionStatus::NoConnection => {
                if *message.ty() != MessageType::Version {
                    return Err(Error::ReceivedWrongMessageType(
                        message.ty().to_string(),
                        MessageType::Version.to_string(),
                    ));
                }
                send_version(&mut stream, &addr, testnet).await?;
                ConnectionStatus::Connecting
            }
            ConnectionStatus::Connecting => {
                if *message.ty() != MessageType::VerAck {
                    return Err(Error::ReceivedWrongMessageType(
                        message.ty().to_string(),
                        MessageType::VerAck.to_string(),
                    ));
                }
                send_verack(&mut stream, testnet).await?;
                info!("Handshake successful with {}", addr);
                ConnectionStatus::Connected
            }
            // If connected accept all the messages
            ConnectionStatus::Connected => ConnectionStatus::Connected,
        };

        connections.insert(addr, new_status);
    }
}

async fn send_version(
    stream: &mut TcpStream,
    addr: &SocketAddr,
    testnet: bool,
) -> Result<(), Error> {
    let version = VersionBuilder::default()
        .receiver_address(*addr)
        .sender_address(stream.local_addr().map_err(Error::LocalAddress)?)
        .build()
        .map_err(Error::BuildVersionPayload)?;
    let message = Message::build(Payload::Version(version), MessageType::Version, testnet)
        .serialize()
        .map_err(Error::BuildMessage)?;

    // Send the serialized message
    stream
        .write_all(&message)
        .await
        .map_err(Error::SendVersion)?;
    stream.flush().await.map_err(Error::FailedToFlushStream)?;

    Ok(())
}

async fn send_verack(stream: &mut TcpStream, testnet: bool) -> Result<(), Error> {
    let verack = VerAck;
    let message = Message::build(Payload::VerAck(verack), MessageType::VerAck, testnet)
        .serialize()
        .map_err(Error::BuildMessage)?;

    // Send the serialized message
    stream
        .write_all(&message)
        .await
        .map_err(Error::SendVerack)?;
    stream.flush().await.map_err(Error::FailedToFlushStream)?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to build the local address")]
    LocalAddress(#[source] std::io::Error),
    #[error("Failed to build the version payload")]
    BuildVersionPayload(#[source] VersionBuilderError),
    #[error("Failed to build the version message")]
    BuildMessage(#[source] SerdeBitcoinError),
    #[error("Failed to send the version message")]
    SendVersion(#[source] std::io::Error),
    #[error("Failed to send the verack message")]
    SendVerack(#[source] std::io::Error),
    #[error("Failed to flush the stream")]
    FailedToFlushStream(#[source] std::io::Error),
    #[error("Failed to fill buffer")]
    FillBuffer(#[source] std::io::Error),
    #[error("Failed to deserialize the version message response")]
    DeserializeVersionResponse(#[source] SerdeBitcoinError),
    #[error("Received wrong message type. Expected {0}, received {1}")]
    ReceivedWrongMessageType(String, String),
    #[error("Failed to get peer address")]
    FailedToGetPeerAddr(#[source] std::io::Error),
}
