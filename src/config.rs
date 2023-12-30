use clap::Parser;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn is_testnet(&self) -> bool {
        match self {
            Network::Mainnet => false,
            Network::Testnet => true,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// Listener configuration
    pub listener: Option<ListenerConfig>,
    /// Sender configuration
    pub sender: Option<SenderConfig>,
}

impl Config {
    pub fn parse(path: &Path) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path).map_err(|_| Error::File(path.into()))?;
        let file = serde_yaml::from_str::<Self>(&content).map_err(|_| Error::File(path.into()))?;
        Ok(file)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListenerConfig {
    /// Target TCP port
    pub port: u16,

    /// Network: mainnet or testnet
    pub network: Network,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SenderConfig {
    /// Bitcoin DNS seed
    pub dns_seed: String,

    /// Target TCP port
    pub port: u16,

    /// Network: mainnet or testnet
    pub network: Network,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read the file {0}")]
    File(Box<Path>),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Sets a custom configuration file
    #[clap(short, long, default_value = "config_files/testnet.yaml")]
    pub config: String,
}
