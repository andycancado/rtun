# SSH Tunnel CLI

A simple command-line tool written in Rust to create SSH tunnels.

## Features

- Create multiple SSH tunnels specified by a list of ports.
- Gracefully handle termination signals (SIGINT, SIGTERM).

## Installation

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.
2. Clone the repository:
    ```sh
    git clone https://github.com/andycancado/rtun.git
    cd rtun
    ```
3. Build the project:
    ```sh
    cargo build --release
    ```

## Usage

Run the CLI with the desired ports, user, and host:

```sh
Usage: rtun [OPTIONS] <PORTS>...

Arguments:
  <PORTS>...  List of ports to tunnel

Options:
      --user <USER>  Remote user [default: user]
      --host <HOST>  Remote host [default: localhost]
  -h, --help         Print help information
  -V, --version      Print version information
```

Example:
```sh
cargo run --release -- 11434 10600 8088 --user user --host localhost
```

Build:
```sh
cargo build --release
```

This command will set up SSH tunnels for the specified ports and block the terminal until you press `Ctrl+C`.

## Example

### Creating SSH Tunnels

To create SSH tunnels for ports 11434, 10600, and 8088 with user `user` and host `localhost`:

```sh
cargo run --release -- 11434 10600 8088 --user user --host localhost
```

### Using Default User and Host

The following command will use the default user (`user`) and host (`localhost`) for the specified ports:

```sh
cargo run --release -- 11434 10600 8088
```

### Specifying User and Host

You can override the defaults with the `--user` and `--host` options:

```sh
cargo run --release -- 11434 10600 8088 --user newuser --host 192.168.1.100
```

## Graceful Shutdown

The CLI tool handles signals such as `SIGINT` (typically sent with `Ctrl+C`) and `SIGTERM` to gracefully terminate the SSH tunnels.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please submit issues or pull requests for any improvements or features you would like to add.

## Acknowledgements

- Built with [Rust](https://www.rust-lang.org/).
