# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BindLocal Server is a high-performance proxy server that enables secure local development tunneling. It acts as a bridge between web browsers and local development environments, allowing external access to local services through HTTP and TCP protocols.

The server operates with two concurrent components:
- **HTTP Server** (default port 8080) - Receives HTTP requests from web browsers
- **TCP Server** (default port 9090) - Establishes socket connections with BindLocal clients

## Build and Development Commands

```bash
# Build release version
cargo build --release

# Run in development mode with logging
cargo run

# Run with custom ports (HTTP port, TCP port)
cargo run -- 3000 4000

# Run release binary
./target/release/bindlocal-server

# Run release binary with custom ports
./target/release/bindlocal-server 3000 4000

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Docker build
docker build . -t bindlocal-server:latest

# Docker run
docker run -p 8080:8080 -p 9090:9090 bindlocal-server:latest
```

## Architecture

### Core Components

The application uses a shared state model with async message passing:

1. **SharedState** (`src/shared.rs`) - Central state management using Arc<Mutex<HashMap>> for concurrent access
   - `tcp_connections`: Maps client IDs to channels for sending HTTP requests to TCP clients
   - `http_connections`: Maps transaction IDs to channels for sending responses back to HTTP clients
   - `subdomains`: Tracks active subdomain registrations to prevent duplicates

2. **HTTP Server** (`src/http_server.rs`) - Handles incoming HTTP requests
   - Parses subdomain from Host header to identify target client
   - Generates unique transaction ID (format: `{client_id}_tx-{random}`)
   - Creates unbounded channel for receiving response from TCP client
   - Waits for TCP client to process request and return response
   - Supports keep-alive connections (checks Connection header)

3. **TCP Server** (`src/tcp_server.rs`) - Manages persistent connections with BindLocal clients
   - Validates client version on connection (minimum version: 0.0.2)
   - Handles subdomain registration (auto-generates if not provided or if duplicate)
   - Receives HTTP requests via channel and forwards to client socket
   - Reads full HTTP response (handles Content-Length and Transfer-Encoding: chunked)
   - Sends response back to waiting HTTP connection via channel

### Communication Flow

1. Browser sends HTTP request to HTTP Server
2. HTTP Server extracts subdomain from Host header to identify client
3. HTTP Server creates transaction ID and registers a response channel
4. HTTP Server sends request to TCP Server via SharedState channel
5. TCP Server forwards request to connected client over socket
6. Client processes request locally and sends HTTP response back over socket
7. TCP Server reads complete response and sends to HTTP Server via transaction channel
8. HTTP Server writes response back to browser

### Important Implementation Details

- The edition in Cargo.toml is set to "2024" (Rust nightly required for Docker build)
- Both servers run concurrently using `tokio::try_join!`
- Transaction IDs prevent response routing errors when multiple requests are in flight
- Client version checking prevents incompatible client connections
- Subdomain collision handling: appends `-{count}` suffix if duplicate detected
- Response parsing handles both Content-Length and chunked Transfer-Encoding
- Special error handling for client app connection failures (CLIENT_ERROR status)

## Testing

Tests are colocated in each module using `#[cfg(test)]`. Key test patterns:
- Unit tests for parsing and validation functions
- Tests verify transaction ID generation format
- Tests validate version checking logic
- Tests ensure subdomain duplicate detection works correctly

When adding tests, follow the existing pattern of testing individual functions rather than integration tests.
