use crate::{SerdeBitcoin, SerdeBitcoinError};

#[derive(Debug, PartialEq)]
pub struct VerAck;

impl SerdeBitcoin for VerAck {
    fn serialize(&self) -> Result<Vec<u8>, SerdeBitcoinError> {
        Ok(vec![])
    }

    fn deserialize(_data: &mut [u8]) -> Result<VerAck, SerdeBitcoinError> {
        Ok(VerAck {})
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version() {
        // Create a Version
        let version = VerAck;

        // Serialize the Version into a Vec<u8>
        let mut serialized_bytes = version.serialize().expect("serialize");

        // Deserialize the bytes back to Version
        let deserialized: VerAck =
            VerAck::deserialize(&mut serialized_bytes.as_mut_slice()).expect("deserialize");

        // Assert that the deserialized value matches the original value
        assert_eq!(deserialized, version);
    }
}
