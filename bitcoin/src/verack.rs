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
    fn test_verack() {
        // Create a VerAck
        let verack = VerAck;

        // Serialize the VerAck into a Vec<u8>
        let mut serialized_bytes = verack.serialize().expect("serialize");

        // Deserialize the bytes back to VerAck
        let deserialized: VerAck =
            VerAck::deserialize(&mut serialized_bytes.as_mut_slice()).expect("deserialize");

        // Assert that the deserialized value matches the original value
        assert_eq!(deserialized, verack);
    }
}
