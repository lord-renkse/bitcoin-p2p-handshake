use crate::config::{Config, SenderConfig};
use clap::Parser;
use dashmap::DashMap;
use futures::future::join_all;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::{lookup_host, TcpListener};
use tokio::task;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

mod config;
mod listener;
mod sender;

const LOCALHOST: &str = "localhost";

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = config::Args::parse();
    let config = Config::parse(&PathBuf::from_str(&args.config).expect("Correct path"))
        .expect("Failed to parse config file");
    let mut handles = Vec::new();

    if let Some(sender_config) = config.sender {
        let addresses = get_socket_addresses(&sender_config).await;
        let network = Arc::new(sender_config.network);
        for address in addresses {
            let network_clone = network.clone();
            let handle = task::spawn(async move {
                match sender::run(&address, network_clone).await {
                    Ok(resp) => info!("Handshake successful with {}", resp.addr()),
                    Err(e) => error!("{e:?}"),
                }
            });
            handles.push(handle);
        }
    }

    // @TODO: The listener could be spawned into a task and then wait for the task
    if let Some(listener_config) = config.listener {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", listener_config.port))
            .await
            .expect("Failed to bind the listener");
        let network = Arc::new(listener_config.network);
        let connections = Arc::new(DashMap::new());

        info!("Accepting connections");
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let network_clone = network.clone();
                let connections_clone = connections.clone();
                tokio::spawn(async move {
                    match listener::run(stream, network_clone, connections_clone).await {
                        Ok(()) => info!("Connection close"),
                        Err(e) => error!("{e:?}"),
                    }
                });
            } else {
                error!("Failed to accept a connection");
            }
        }
    }

    // Ignore the errors here on purpose
    let _ = join_all(handles).await;
}

async fn get_socket_addresses(config: &SenderConfig) -> Vec<SocketAddr> {
    // @TODO: This is definitely not elegant neither nice, the type of connection should be an enum
    if config.dns_seed.contains(LOCALHOST) {
        vec![SocketAddr::new(
            Ipv4Addr::new(127, 0, 0, 1).into(),
            config.port,
        )]
    } else {
        // @TODO: It shouldn't panic here, the error should be propagated accordingly
        lookup_host((config.dns_seed.clone(), config.port))
            .await
            .expect("host not found")
            .collect()
    }
}
