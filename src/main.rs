mod http_server;
mod request;
mod response;
mod shared;
mod tcp_server;

use http_server::HttpServer;

use shared::SharedState;
use std::env;
use tcp_server::TcpServer;

use tracing_subscriber;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt().with_target(false).init();

    let args: Vec<String> = env::args().collect();
    let http_port = if args.len() > 1 {
        args[1].parse::<u16>().unwrap_or(8080)
    } else {
        8080
    };

    let tcp_port = if args.len() > 2 {
        args[2].parse::<u16>().unwrap_or(9090)
    } else {
        9090
    };

    let http_addr = format!("0.0.0.0:{}", http_port);
    let tcp_addr = format!("0.0.0.0:{}", tcp_port);

    tracing::info!("Starting servers..");

    tracing::info!("HTTP Server will run on http://{}", http_addr);
    tracing::info!("TCP Server will run on tcp://{}", tcp_addr);

    let share_state = SharedState::new();

    // Start both servers concurrently
    let http_server = HttpServer::new(&http_addr, share_state.clone()).await?;
    let tcp_server = TcpServer::new(&tcp_addr, share_state.clone()).await?;

    // Run both servers simultaneously
    tokio::try_join!(http_server.run(), tcp_server.run())?;

    Ok(())
}
