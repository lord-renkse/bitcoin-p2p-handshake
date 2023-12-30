# Bitcoin P2P handshake

## Considerations

- The program is configured with a configuration file in `yaml` format
- The errors are propagated accordingly except the ones triggers during startup
- The program can be run as a sender and connect it to the real testnet/mainnet, or it can be run as a standalone node in localhost
- The types for the bitcoin handshake were defined in an independent crate, so it is properly encapsulated and it can be reused in any other project
- There are basic unit tests specially for the bitcoin types, for the node there aren't unit test. It is something that definitely could be improved
- The localhost address can be managed in a better way, `dns_seed` in the configuration should be an enum
- No library related to bitcoin or p2p handshake were used

## Connecting node to the testnet

To run it in the testnet
```console
cargo run --release -- --config=config_files/testnet.yaml
```

```console
$ cargo run --release -- --config=config_files/testnet.yaml
2023-12-30T10:53:22.466856Z  INFO bitcoin_p2p::sender: Connecting to 122.248.200.20:18333
2023-12-30T10:53:22.466860Z  INFO bitcoin_p2p::sender: Connecting to 51.210.220.135:18333
2023-12-30T10:53:22.466899Z  INFO bitcoin_p2p::sender: Connecting to 72.211.1.222:18333
2023-12-30T10:53:22.466912Z  INFO bitcoin_p2p::sender: Connecting to 13.229.104.97:18333
2023-12-30T10:53:22.466934Z  INFO bitcoin_p2p::sender: Connecting to 51.210.208.202:18333
2023-12-30T10:53:22.466854Z  INFO bitcoin_p2p::sender: Connecting to 132.145.213.181:18333
[...]
2023-12-30T10:53:22.515045Z ERROR bitcoin_p2p: DeserializeVerackResponse(UnknownType("wtxidrelay"))
2023-12-30T10:53:22.517988Z ERROR bitcoin_p2p: DeserializeVersionResponse(IoError(Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }))
2023-12-30T10:53:22.525882Z  INFO bitcoin_p2p: Handshake successful with 188.40.164.205:18333
```

## Connecting node to the mainnet

To run it in the mainnet
```console
cargo run --release -- --config=config_files/mainnet.yaml
```

```console
$ cargo run --release -- --config=config_files/mainnet.yaml
2023-12-30T10:46:40.979715Z  INFO bitcoin_p2p::sender: Connecting to 123.202.193.121:8333
2023-12-30T10:46:40.979736Z  INFO bitcoin_p2p::sender: Connecting to 18.191.254.86:8333
2023-12-30T10:46:40.979719Z  INFO bitcoin_p2p::sender: Connecting to 104.189.105.88:8333
2023-12-30T10:46:40.979714Z  INFO bitcoin_p2p::sender: Connecting to 2.56.90.100:8333
2023-12-30T10:46:40.979759Z  INFO bitcoin_p2p::sender: Connecting to 170.78.215.212:8333
[...]
023-12-30T10:46:40.980116Z ERROR bitcoin_p2p: TcpConnection("[2400:6180:100:d0::a05:5001]:8333", Os { code: 101, kind: NetworkUnreachable, message: "Network is unreachable" })
2023-12-30T10:46:40.980296Z ERROR bitcoin_p2p: TcpConnection("[2001:638:a000:4140::ffff:191]:8333", Os { code: 101, kind: NetworkUnreachable, message: "Network is unreachable" })
2023-12-30T10:46:41.031489Z  INFO bitcoin_p2p: Handshake successful with 185.156.154.129:8333
2023-12-30T10:46:41.052825Z ERROR bitcoin_p2p: DeserializeVerackResponse(UnknownType("sendaddrv2"))
2023-12-30T10:46:41.060822Z  INFO bitcoin_p2p: Handshake successful with 95.216.242.49:8333
```

## Running both nodes locally

To run the listener node
```console
cargo run --release -- --config=config_files/localhost_listener.yaml
```

```console
$ cargo run --release -- --config=config_files/localhost_listener.yaml
    Finished release [optimized] target(s) in 0.18s
     Running `target/release/bitcoin-p2p --config=config_files/localhost_listener.yaml`
2023-12-30T10:49:13.734832Z  INFO bitcoin_p2p: Accepting connections
2023-12-30T10:49:39.141332Z  INFO bitcoin_p2p::listener: Handshake successful with 127.0.0.1:32916
2023-12-30T10:49:39.141396Z  INFO bitcoin_p2p: Connection close

```

To run the sender node (listener node must be running first)
```console
cargo run --release -- --config=config_files/localhost_sender.yaml
```

```console
$ cargo run --release -- --config=config_files/localhost_sender.yaml
    Finished release [optimized] target(s) in 0.04s
     Running `target/release/bitcoin-p2p --config=config_files/localhost_sender.yaml`
2023-12-30T10:49:39.141007Z  INFO bitcoin_p2p::sender: Connecting to 127.0.0.1:8333
2023-12-30T10:49:39.141360Z  INFO bitcoin_p2p: Handshake successful with 127.0.0.1:8333
```
