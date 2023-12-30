use crate::{SerdeBitcoin, SerdeBitcoinError};
use strum_macros::{AsRefStr, Display, EnumString};

#[derive(Clone, Debug, Display, AsRefStr, PartialEq, EnumString)]
pub enum MessageType {
    #[strum(serialize = "version")]
    Version,
    #[strum(serialize = "verack")]
    VerAck,
    #[strum(serialize = "ping")]
    Ping,
    #[strum(serialize = "addr")]
    Addr,
    #[strum(serialize = "getdata")]
    GetData,
    #[strum(serialize = "tx")]
    Tx,
    #[strum(serialize = "block")]
    Block,
    #[strum(serialize = "getblocks")]
    GetBlocks,
    #[strum(serialize = "getheaders")]
    GetHeaders,
    #[strum(serialize = "headers")]
    Headers,
    #[strum(serialize = "reject")]
    Reject,
    #[strum(serialize = "mempool")]
    MemPool,
    #[strum(serialize = "feefilter")]
    FeeFilter,
    #[strum(serialize = "sendheaders")]
    SendHeaders,
    #[strum(serialize = "sendcmpct")]
    SendCmpct,
    #[strum(serialize = "wtxidrelay")]
    WtxIdRelay,
    #[strum(serialize = "sendaddrv2")]
    SendAddrV2,
}

impl MessageType {
    const SIZE: usize = 12;
}

impl SerdeBitcoin for MessageType {
    fn serialize(&self) -> Result<Vec<u8>, SerdeBitcoinError> {
        let mut result = self.to_string().into_bytes();

        if result.len() > MessageType::SIZE {
            return Err(SerdeBitcoinError::MessageTypeTooLong(result.len()));
        }

        result.append(&mut vec![0; MessageType::SIZE - result.len()]);
        Ok(result)
    }

    fn deserialize(data: &mut [u8]) -> Result<MessageType, SerdeBitcoinError> {
        if data.len() > MessageType::SIZE {
            return Err(SerdeBitcoinError::MessageTypeTooLong(data.len()));
        }

        // Calculate the number of zeroes needed to fill the remaining space
        let zeroes_count = data.iter().rev().take_while(|&&byte| byte == 0).count();

        // Create the MessageType variant
        if let Some(version) = data.get(0..(data.len() - zeroes_count)) {
            let version_str = String::from_utf8_lossy(version);
            version_str
                .parse::<MessageType>()
                .map_err(|_| SerdeBitcoinError::UnknownType(version_str.to_string()))
        } else {
            Err(SerdeBitcoinError::Infallible)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_message_type_version() {
        // Create a MessageType::Version
        let message_type = MessageType::Version;

        // Serialize the MessageType into a Vec<u8>
        let mut serialized_bytes = message_type.serialize().expect("serialize");

        let expected = "version\0\0\0\0\0";

        // Convert the expected string to bytes
        let expected_bytes = expected.as_bytes();

        // Assert that the serialized bytes length is as expected
        assert_eq!(serialized_bytes.len(), MessageType::SIZE);

        // Assert that the serialized bytes match the expected bytes
        assert_eq!(serialized_bytes, expected_bytes);

        // Deserialize the bytes back to MessageType
        let deserialized: MessageType =
            MessageType::deserialize(&mut serialized_bytes.as_mut_slice()).expect("deserialize");

        // Assert that the deserialized value matches the original value
        assert_eq!(deserialized, message_type);
    }
}
