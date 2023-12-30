use crate::config::Network;
use bitcoin::message_type::MessageType;
use bitcoin::verack::VerAck;
use bitcoin::version::{VersionBuilder, VersionBuilderError};
use bitcoin::{Message, Payload, SerdeBitcoin, SerdeBitcoinError};
use getset::Getters;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::error::Elapsed;
use tokio::time::timeout;
use tracing::{error, info};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(15);
const VERSION_TIMEOUT: Duration = Duration::from_secs(30);
const VERACK_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Getters)]
// @TODO: Add more fields from the node response as needed
pub struct ConnectionInfo {
    #[getset(get = "pub")]
    addr: SocketAddr,
}

pub async fn run(addr: &SocketAddr, network: Arc<Network>) -> Result<ConnectionInfo, Error> {
    info!("Connecting to {addr}");
    let mut stream = timeout(CONNECTION_TIMEOUT, TcpStream::connect(addr))
        .await
        .map_err(Error::ConnectionTimeout)?
        .map_err(|e| Error::TcpConnection(addr.to_string(), e))?;

    let testnet = network.is_testnet();
    // @TODO: Improvement: To add a retry mechanism
    let resp_version = timeout(VERSION_TIMEOUT, version(&mut stream, addr, testnet))
        .await
        .map_err(Error::VersionTimeout)??;

    if *resp_version.ty() != MessageType::Version {
        return Err(Error::ReceivedWrongMessageType(
            resp_version.ty().to_string(),
            MessageType::Version.to_string(),
        ));
    }
    stream.flush().await.map_err(Error::FailedToFlushStream)?;

    let resp_verack = timeout(VERACK_TIMEOUT, verack(&mut stream, testnet))
        .await
        .map_err(Error::VerackTimeout)??;
    if *resp_verack.ty() != MessageType::VerAck {
        return Err(Error::ReceivedWrongMessageType(
            resp_verack.ty().to_string(),
            MessageType::VerAck.to_string(),
        ));
    }
    Ok(ConnectionInfo { addr: *addr })
}

async fn version(
    stream: &mut TcpStream,
    addr: &SocketAddr,
    testnet: bool,
) -> Result<Message, Error> {
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

    // Read the response
    let mut br = BufReader::new(stream);
    let mut response_buffer = br.fill_buf().await.map_err(Error::FillBuffer)?.to_vec();

    // Deserialize the response
    Message::deserialize(&mut response_buffer).map_err(Error::DeserializeVersionResponse)
}

async fn verack(stream: &mut TcpStream, testnet: bool) -> Result<Message, Error> {
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

    // Read the response
    let mut br = BufReader::new(stream);
    let mut response_buffer = br.fill_buf().await.map_err(Error::FillBuffer)?.to_vec();

    // Deserialize the response
    Message::deserialize(&mut response_buffer).map_err(Error::DeserializeVerackResponse)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to connect to {0}")]
    TcpConnection(String, #[source] std::io::Error),
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
    #[error("Failed to deserialize the verack message response")]
    DeserializeVerackResponse(#[source] SerdeBitcoinError),
    #[error("Version timeout")]
    VersionTimeout(#[source] Elapsed),
    #[error("Verack timeout")]
    VerackTimeout(#[source] Elapsed),
    #[error("Connection timeout")]
    ConnectionTimeout(#[source] Elapsed),
    #[error("Received wrong message type. Expected {0}, received {1}")]
    ReceivedWrongMessageType(String, String),
}
