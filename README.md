# Tonik

A Rust library and command line interface for Teltonika routers.

## Library

The library provides a set of functions to interact with Teltonika routers using the REST API.

### Installation

Add the library to your `Cargo.toml` file.

```toml
[dependencies]
tonik = "0.1"
```

### Usage

Futher information can be found in the [documentation](https://docs.rs/tonik).

```rust
use tonik::Client;

#[tokio::main]
async fn main() {
    let client = Client::new("ROUTER_IP");

    // Authenticate
    client.authenticate("USERNAME", "PASSWORD").await.unwrap();

    // Get device information
    let info = client.get_device_info().await.unwrap();
    println!("{:?}", info);
}
```

## Command Line Interface

The CLI is a command line tool for Teltonika routers. It provides a set of commands to interact with the router using the REST API.

### Installation

Install the CLI using `cargo` package manager included with Rust.

```bash
cargo install tonik-cli
```

### Usage

```
Usage: tonik [OPTIONS] --password <PASSWORD> <COMMAND>

Commands:
  dhcp      DHCP related commands
  firmware  Firmware related commands
  gps       Global Positioning System related commands
  help      Print this message or the help of the given subcommand(s)

Options:
      --host <HOST>          Teltonika host [env: TELTONIKA_HOST=] [default: 192.168.7.1]
      --username <USERNAME>  Teltonika username [env: TELTONIKA_USERNAME=] [default: admin]
      --password <PASSWORD>  Teltonika password [env: TELTONIKA_PASSWORD=]
      --json                 Output in JSON format
  -h, --help                 Print help
```

### Examples

#### List Connected MAC Addresses

```
$ tonik --json dhcp ipv4 status | jq -r '.[].macaddr'
00:01:02:03:04:05
EA:EB:EC:ED:EE:EF
```

#### Get GPS Information

```
$ tonik gps position
Accuracy: 0.8
Fix status: 1
Altitude: 6.2
Timestamp: 1714074213
Satellites: 7
Longitude: -0.054569
Latitude: 51.589495
Angle: 0
UTC timestamp: 1714074213
```
