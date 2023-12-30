pub mod message_type;
pub mod verack;
pub mod version;

use crate::message_type::MessageType;
use crate::verack::VerAck;
use crate::version::Version;
use byteorder::{LittleEndian, ReadBytesExt};
use getset::Getters;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::num::TryFromIntError;
use std::string::FromUtf8Error;
use thiserror::Error;

pub trait SerdeBitcoin {
    fn serialize(&self) -> Result<Vec<u8>, SerdeBitcoinError>;
    fn deserialize(data: &mut [u8]) -> Result<Self, SerdeBitcoinError>
    where
        Self: Sized;
}

#[derive(Error, Debug)]
pub enum SerdeBitcoinError {
    #[error("Message type too long: {0}")]
    MessageTypeTooLong(usize),
    #[error("It can never fail")]
    Infallible,
    #[error("Unknown/unsupported message type: {0}")]
    UnknownType(String),
    #[error("Io Error")]
    IoError(#[from] std::io::Error),
    #[error("Invalid user agent length")]
    InvalidUserAgentLength(#[source] TryFromIntError),
    #[error("Invalid payload length")]
    InvalidPayloadLength(#[source] TryFromIntError),
    #[error("Failed to parse user agent")]
    FailedToParseUserAgent(#[source] FromUtf8Error),
    #[error("Failed to map to IPv4")]
    FailedToMapToIpv4,
    #[error("Invalid checksum")]
    InvalidChecksum,
}

/// Magic bytes for mainnet
const MAGIC_BYTES_MAINNET: [u8; 4] = [0xf9, 0xbe, 0xb4, 0xd9];

/// Magic bytes for testnet
const MAGIC_BYTES_TESTNET: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];

/// Magic bytes Size
const MAGIC_BYTES_LENGTH: usize = 4;
/// Checksum Size
const CHECKSUM_LENGTH: usize = 4;

#[derive(Debug, PartialEq)]
pub enum Payload {
    Version(Version),
    VerAck(VerAck),
}

impl Payload {
    fn serialize(&self) -> Result<Vec<u8>, SerdeBitcoinError> {
        match self {
            Payload::Version(version) => version.serialize(),
            Payload::VerAck(verack) => verack.serialize(),
        }
    }
}

#[derive(Getters, Debug, PartialEq)]
pub struct Message {
    #[getset(get = "pub")]
    magic_bytes: [u8; MAGIC_BYTES_LENGTH],

    #[getset(get = "pub")]
    ty: MessageType,

    #[getset(get = "pub")]
    payload: Payload,
}

impl Message {
    const BASE_SIZE: usize = 24;

    pub fn build(payload: Payload, ty: MessageType, testet: bool) -> Self {
        let magic_bytes = if testet {
            MAGIC_BYTES_TESTNET
        } else {
            MAGIC_BYTES_MAINNET
        };

        Self {
            magic_bytes,
            ty,
            payload,
        }
    }

    fn build_checksum(payload: &[u8]) -> [u8; CHECKSUM_LENGTH] {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let first_hash = hasher.finalize();

        hasher = Sha256::new();
        hasher.update(&first_hash);
        let second_hash = hasher.finalize();

        // @TODO: Remove the panic from here, it should never panic but it is better to propagate the error and handle it properly
        let checksum: [u8; CHECKSUM_LENGTH] = second_hash[..CHECKSUM_LENGTH]
            .try_into()
            .expect("Wrong length for checksum");

        checksum
    }
}

impl SerdeBitcoin for Message {
    fn serialize(&self) -> Result<Vec<u8>, SerdeBitcoinError> {
        // Payload
        let payload_bytes = self.payload.serialize()?;
        let payload_length =
            u32::try_from(payload_bytes.len()).map_err(SerdeBitcoinError::InvalidPayloadLength)?;

        let mut result = Vec::with_capacity(Message::BASE_SIZE + payload_bytes.len());

        // Magic Bytes
        result.extend_from_slice(&self.magic_bytes);

        // Message type
        let message_type = self.ty.serialize()?;
        result.extend_from_slice(&message_type);

        // Payload Length
        result.extend_from_slice(&payload_length.to_le_bytes());

        // Checksum
        let checksum = Self::build_checksum(&payload_bytes);
        result.extend_from_slice(&checksum);

        // Payload
        result.extend_from_slice(&payload_bytes);

        Ok(result)
    }

    fn deserialize(data: &mut [u8]) -> Result<Self, SerdeBitcoinError>
    where
        Self: Sized,
    {
        let mut cursor = Cursor::new(data);

        // Read Magic Bytes
        let mut magic_bytes = [0u8; MAGIC_BYTES_LENGTH];
        cursor.read_exact(&mut magic_bytes)?;

        // Message type
        let mut message_type_bytes = [0u8; 12];
        cursor.read_exact(&mut message_type_bytes)?;
        let message_type = MessageType::deserialize(&mut message_type_bytes)?;

        // Read Payload Length
        let payload_length = cursor.read_u32::<LittleEndian>()?;

        // Read Checksum
        let mut checksum = [0u8; 4];
        cursor.read_exact(&mut checksum)?;

        // Read Payload
        let mut payload_bytes = vec![
            0u8;
            usize::try_from(payload_length)
                .map_err(SerdeBitcoinError::InvalidPayloadLength)?
        ];
        cursor.read_exact(&mut payload_bytes)?;

        // Validate Payload
        if Self::build_checksum(&payload_bytes)[..] != checksum {
            return Err(SerdeBitcoinError::InvalidChecksum);
        }

        // Deserialize Payload
        let payload = match message_type {
            MessageType::Version => Payload::Version(Version::deserialize(&mut payload_bytes)?),
            MessageType::VerAck => Payload::VerAck(VerAck::deserialize(&mut payload_bytes)?),
            ty => return Err(SerdeBitcoinError::UnknownType(ty.to_string())),
        };

        Ok(Message {
            magic_bytes,
            ty: message_type,
            payload,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::version::VersionBuilder;
    use std::net::SocketAddr;

    #[test]
    fn test_version() {
        // Create a Version
        let version = VersionBuilder::default()
            .receiver_address("127.0.0.1:18333".parse::<SocketAddr>().unwrap())
            .sender_address("127.0.0.1:18334".parse::<SocketAddr>().unwrap())
            .build()
            .unwrap();

        let message = Message::build(Payload::Version(version), MessageType::Version, true);

        // Serialize the Message into a Vec<u8>
        let mut serialized_bytes = message.serialize().expect("serialize");

        // Deserialize the bytes back to Message
        let deserialized: Message =
            Message::deserialize(&mut serialized_bytes.as_mut_slice()).expect("deserialize");

        // Assert that the deserialized value matches the original value
        assert_eq!(deserialized, message);
    }

    #[test]
    fn test_verack() {
        // Create a Version
        let verack = VerAck;

        let message = Message::build(Payload::VerAck(verack), MessageType::VerAck, true);

        // Serialize the Message into a Vec<u8>
        let mut serialized_bytes = message.serialize().expect("serialize");

        // Deserialize the bytes back to Message
        let deserialized: Message =
            Message::deserialize(&mut serialized_bytes.as_mut_slice()).expect("deserialize");

        // Assert that the deserialized value matches the original value
        assert_eq!(deserialized, message);
    }
}
