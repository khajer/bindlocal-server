mod http_server;
mod request;
mod response;
mod shared;
mod tcp_server;

use http_server::HttpServer;
use shared::SharedState;
use std::env;
use tcp_server::TcpServer;
use tracing::info;
use tracing_subscriber::fmt;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub http_port: u16,
    pub tcp_port: u16,
    pub http_addr: String,
    pub tcp_addr: String,
}

impl ServerConfig {
    pub fn from_args() -> Result<Self, Box<dyn std::error::Error>> {
        let args: Vec<String> = env::args().collect();

        let http_port = if args.len() > 1 {
            args[1]
                .parse::<u16>()
                .map_err(|_| format!("Invalid HTTP port: {}", args[1]))?
        } else {
            8080
        };

        let tcp_port = if args.len() > 2 {
            args[2]
                .parse::<u16>()
                .map_err(|_| format!("Invalid TCP port: {}", args[2]))?
        } else {
            9090
        };

        Ok(ServerConfig {
            http_addr: format!("0.0.0.0:{}", http_port),
            tcp_addr: format!("0.0.0.0:{}", tcp_port),
            http_port,
            tcp_port,
        })
    }
}

fn setup_logging() {
    fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();
}

fn print_startup_info(config: &ServerConfig) {
    info!("Starting servers..");
    info!("HTTP Server will run on http://{}", config.http_addr);
    info!("TCP Server will run on tcp://{}", config.tcp_addr);
}

async fn initialize_servers(
    config: &ServerConfig,
    shared_state: SharedState,
) -> Result<(HttpServer, TcpServer), Box<dyn std::error::Error>> {
    let http_server = HttpServer::new(&config.http_addr, shared_state.clone()).await?;
    let tcp_server = TcpServer::new(&config.tcp_addr, shared_state.clone()).await?;

    Ok((http_server, tcp_server))
}

async fn run_servers(
    http_server: HttpServer,
    tcp_server: TcpServer,
) -> Result<(), Box<dyn std::error::Error>> {
    tokio::try_join!(http_server.run(), tcp_server.run())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::from_args()?;
    setup_logging();
    print_startup_info(&config);

    let shared_state = SharedState::new();

    let (http_server, tcp_server) = initialize_servers(&config, shared_state).await?;

    run_servers(http_server, tcp_server).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default_ports() {
        let config = ServerConfig {
            http_port: 8080,
            tcp_port: 9090,
            http_addr: "0.0.0.0:8080".to_string(),
            tcp_addr: "0.0.0.0:9090".to_string(),
        };

        assert_eq!(config.http_port, 8080);
        assert_eq!(config.tcp_port, 9090);
        assert_eq!(config.http_addr, "0.0.0.0:8080");
        assert_eq!(config.tcp_addr, "0.0.0.0:9090");
    }

    #[test]
    fn test_server_config_custom_ports() {
        let config = ServerConfig {
            http_port: 3000,
            tcp_port: 4000,
            http_addr: "0.0.0.0:3000".to_string(),
            tcp_addr: "0.0.0.0:4000".to_string(),
        };

        assert_eq!(config.http_port, 3000);
        assert_eq!(config.tcp_port, 4000);
        assert_eq!(config.http_addr, "0.0.0.0:3000");
        assert_eq!(config.tcp_addr, "0.0.0.0:4000");
    }
}
