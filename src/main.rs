mod http_server;
mod request;
mod response;
mod router;
mod tcp_server;

use http_server::HttpServer;
use std::env;
use tcp_server::TcpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get ports from command line arguments or use defaults
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

    let http_addr = format!("127.0.0.1:{}", http_port);
    let tcp_addr = format!("127.0.0.1:{}", tcp_port);

    println!("Starting servers...");
    println!("HTTP Server will run on http://{}", http_addr);
    println!("TCP Server will run on tcp://{}", tcp_addr);

    // Start both servers concurrently
    let http_server = HttpServer::new(&http_addr).await?;
    let tcp_server = TcpServer::new(&tcp_addr).await?;

    // Run both servers simultaneously
    tokio::try_join!(http_server.run(), tcp_server.run())?;

    Ok(())
}
