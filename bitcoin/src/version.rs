use crate::{SerdeBitcoin, SerdeBitcoinError};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use chrono::Utc;
use derive_builder::Builder;
use getset::Getters;
use rand::random;
use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

// @TODO: Majority of these defaults should be part of the configuration and not hard-coded here
#[derive(Builder, Getters, Debug, PartialEq)]
#[builder(setter(into))]
pub struct Version {
    #[getset(get = "pub")]
    #[builder(default = "70016")]
    protocol_version: i32,

    #[getset(get = "pub")]
    #[builder(default = "1")]
    services: u64,

    #[getset(get = "pub")]
    #[builder(default = "Utc::now().timestamp()")]
    timestamp: i64,

    #[getset(get = "pub")]
    #[builder(default = "1")]
    receiver_services: u64,

    #[getset(get = "pub")]
    receiver_address: SocketAddr,

    #[getset(get = "pub")]
    #[builder(default = "1")]
    sender_services: u64,

    #[getset(get = "pub")]
    sender_address: SocketAddr,

    #[getset(get = "pub")]
    #[builder(default = "random::<u64>()")]
    nonce: u64,

    #[getset(get = "pub")]
    #[builder(default = "\"Satoshi:0.21.0\".to_string()")]
    user_agent: String,
    #[getset(get = "pub")]
    #[builder(default = "0")]
    start_height: i32,

    #[getset(get = "pub")]
    #[builder(default = "false")]
    relay: bool,
}

impl Version {
    const SIZE: usize = 100;
}

// @TODO: This obviously panics if out of range. Fix it.
fn is_ipv4_mapped_ipv6(addr: &Ipv6Addr) -> bool {
    let segments = addr.segments();
    segments[0..5] == [0, 0, 0, 0, 0] && segments[5] == 0xffff
}

impl SerdeBitcoin for Version {
    fn serialize(&self) -> Result<Vec<u8>, SerdeBitcoinError> {
        let mut result = Vec::with_capacity(Version::SIZE);
        result.write_i32::<LittleEndian>(self.protocol_version)?;
        result.write_u64::<LittleEndian>(self.services)?;
        result.write_i64::<LittleEndian>(self.timestamp)?;

        result.write_u64::<LittleEndian>(self.receiver_services)?;
        result.write_u128::<BigEndian>(u128::from_be_bytes(
            match self.receiver_address.ip() {
                IpAddr::V4(x) => x.to_ipv6_mapped(),
                IpAddr::V6(x) => x,
            }
            .octets(),
        ))?;
        result.write_u16::<BigEndian>(self.receiver_address.port())?;

        result.write_u64::<LittleEndian>(self.sender_services)?;
        result.write_u128::<BigEndian>(u128::from_be_bytes(
            match self.sender_address.ip() {
                IpAddr::V4(x) => x.to_ipv6_mapped(),
                IpAddr::V6(x) => x,
            }
            .octets(),
        ))?;
        result.write_u16::<BigEndian>(self.sender_address.port())?;

        result.write_u64::<LittleEndian>(self.nonce)?;
        result.write_u8(
            u8::try_from(self.user_agent().len())
                .map_err(SerdeBitcoinError::InvalidUserAgentLength)?,
        )?;
        result.write_all(self.user_agent().as_bytes())?;
        result.write_i32::<LittleEndian>(self.start_height)?;
        result.write_u8(self.relay.into())?;

        Ok(result)
    }

    fn deserialize(data: &mut [u8]) -> Result<Version, SerdeBitcoinError> {
        let mut cursor = Cursor::new(data);
        let protocol_version = cursor.read_i32::<LittleEndian>()?;
        let services = cursor.read_u64::<LittleEndian>()?;
        let timestamp = cursor.read_i64::<LittleEndian>()?;

        let receiver_services = cursor.read_u64::<LittleEndian>()?;
        let receiver_ip: Ipv6Addr = cursor.read_u128::<BigEndian>()?.into();
        let receiver_ip = if is_ipv4_mapped_ipv6(&receiver_ip) {
            IpAddr::V4(
                receiver_ip
                    .to_ipv4_mapped()
                    .ok_or(SerdeBitcoinError::FailedToMapToIpv4)?,
            )
        } else {
            IpAddr::V6(receiver_ip)
        };
        let receiver_port = cursor.read_u16::<BigEndian>()?;
        let receiver_address = SocketAddr::new(receiver_ip, receiver_port);

        let sender_services = cursor.read_u64::<LittleEndian>()?;
        let sender_ip: Ipv6Addr = cursor.read_u128::<BigEndian>()?.into();
        let sender_ip = if is_ipv4_mapped_ipv6(&sender_ip) {
            IpAddr::V4(
                sender_ip
                    .to_ipv4_mapped()
                    .ok_or(SerdeBitcoinError::FailedToMapToIpv4)?,
            )
        } else {
            IpAddr::V6(sender_ip)
        };
        let sender_port = cursor.read_u16::<BigEndian>()?;
        let sender_address = SocketAddr::new(sender_ip, sender_port);

        let nonce = cursor.read_u64::<LittleEndian>()?;
        let user_agent_len = cursor.read_u8()?;
        let mut user_agent_bytes = vec![0u8; usize::from(user_agent_len)];
        cursor.read_exact(&mut user_agent_bytes)?;
        let user_agent = String::from_utf8(user_agent_bytes)
            .map_err(SerdeBitcoinError::FailedToParseUserAgent)?;
        let start_height = cursor.read_i32::<LittleEndian>()?;
        let relay = cursor.read_u8()? != 0;

        Ok(Version {
            protocol_version,
            services,
            timestamp,
            receiver_services,
            receiver_address,
            sender_services,
            sender_address,
            nonce,
            user_agent,
            start_height,
            relay,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version() {
        // Create a Version
        let version = VersionBuilder::default()
            .receiver_address("127.0.0.1:18333".parse::<SocketAddr>().unwrap())
            .sender_address("127.0.0.1:18334".parse::<SocketAddr>().unwrap())
            .build()
            .unwrap();

        // Serialize the Version into a Vec<u8>
        let mut serialized_bytes = version.serialize().expect("serialize");

        // Assert that the serialized bytes length is as expected
        assert_eq!(serialized_bytes.len(), Version::SIZE);

        // Deserialize the bytes back to Version
        let deserialized: Version =
            Version::deserialize(&mut serialized_bytes.as_mut_slice()).expect("deserialize");

        // Assert that the deserialized value matches the original value
        assert_eq!(deserialized, version);
    }
}
