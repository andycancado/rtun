<p align="center"> <img src="https://i.ibb.co/GRkCKJV/rtun-logo2-transformed.png"/></p>

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
Usage: rtun 

```

Example:
```sh
cargo run --release 
```

Build:
```sh
cargo build --release
```

This command will set up SSH tunnels for the specified ports and block the terminal until you press `Ctrl+C`.

## Graceful Shutdown

The CLI tool handles signals such as `SIGINT` (typically sent with `Ctrl+C`) and `SIGTERM` to gracefully terminate the SSH tunnels.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please submit issues or pull requests for any improvements or features you would like to add.

## Acknowledgements

- Built with [Rust](https://www.rust-lang.org/).
