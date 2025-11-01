# BindLocal Server

A high-performance proxy server application that enables secure local development tunneling. The server acts as a bridge between web browsers and local development environments, allowing external access to local services through HTTP and TCP protocols.

## Overview

BindLocal Server consists of two main components working in tandem:

1. **HTTP Server** - Receives HTTP requests from web browsers
2. **TCP Server** - Establishes socket connections with BindLocal clients

This architecture enables bidirectional communication between external web browsers and local development services.

## Architecture Flow

### Step 1: Client to Browser
```
Web Browser → BindLocal Server → BindLocal Client → Localhost (Dev)
```

### Step 2: Browser to Client
```
Localhost (Dev) → BindLocal Client → BindLocal Server → Web Browser
```


## Quick Start

### Prerequisites

- Rust 1.70+ (or use Docker)
- Git

### Installation

#### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/khajer/bindlocal-server
cd bindlocal-server

# Build the project in release mode
cargo build --release

# Run the server
./target/release/bindlocal-server
```

#### Option 2: Docker

```bash
# Build the Docker image
docker build . -t bindlocal-server:latest

# Run the container
docker run -p 8080:8080 -p 9090:9090 bindlocal-server:latest
```

### Usage

The server accepts optional command-line arguments for custom ports:

```bash
# Use default ports (HTTP: 8080, TCP: 9090)
./bindlocal-server

# Use custom ports
./bindlocal-server <HTTP_PORT> <TCP_PORT>
./bindlocal-server 3000 4000  # HTTP on 3000, TCP on 4000
```

### Running in Development

```bash
# Run with logging enabled
cargo run

# Run with custom ports
cargo run -- 3000 4000
```

## Configuration

The server configuration includes:

- **HTTP Port**: Default `8080` - for incoming web browser requests
- **TCP Port**: Default `9090` - for client socket connections
- **Bind Address**: `0.0.0.0` - listens on all network interfaces

## Development Status

- [✅] React application testing
- [✅] Svelte application testing
- [🔄] WebSocket support
- [🔄] HTTPS/SSL termination
- [🔄] Load balancing
- [🔄] Rate limiting


## Official Website

[https://connl.io](https://connl.io)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
