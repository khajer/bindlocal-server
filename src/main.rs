mod request;
mod response;
mod router;
mod server;

use server::HttpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = HttpServer::new("127.0.0.1:8080").await?;
    println!("HTTP Server running on http://127.0.0.1:8080");
    server.run().await
}
