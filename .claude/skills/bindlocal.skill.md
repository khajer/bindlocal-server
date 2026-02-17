# bindlocal

BindLocal Server development helper commands for building, running, and testing the proxy server.

## Commands

### run
Run the BindLocal server in development mode with logging

```bash
cargo run
```

### run-custom
Run the BindLocal server with custom HTTP and TCP ports

Example: `run-custom 3000 4000` runs HTTP on port 3000 and TCP on port 4000

```bash
cargo run -- {{args}}
```

### build
Build the release version of BindLocal server

```bash
cargo build --release
```

### run-release
Run the release binary

```bash
./target/release/bindlocal-server
```

### test
Run all tests

```bash
cargo test
```

### test-verbose
Run tests with output visible

```bash
cargo test -- --nocapture
```

### test-specific
Run a specific test by name

Example: `test-specific test_name`

```bash
cargo test {{args}}
```

### docker-build
Build Docker image for BindLocal server

```bash
docker build . -t bindlocal-server:latest
```

### docker-run
Run BindLocal server in Docker container

```bash
docker run -p 8080:8080 -p 9090:9090 bindlocal-server:latest
```

### check
Run cargo check to verify code compiles

```bash
cargo check
```

### clean
Clean build artifacts

```bash
cargo clean
```

### clippy
Run clippy for linting

```bash
cargo clippy -- -D warnings
```

### fmt
Format code using rustfmt

```bash
cargo fmt
```

## Usage Examples

- `/bindlocal run` - Start server in dev mode (HTTP:8080, TCP:9090)
- `/bindlocal run-custom 3000 4000` - Start with custom ports
- `/bindlocal build` - Build release binary
- `/bindlocal test` - Run all tests
- `/bindlocal test-specific version_check` - Run specific test
- `/bindlocal docker-build` - Build Docker image
- `/bindlocal docker-run` - Run in container
