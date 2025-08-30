use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::request::HttpRequest;
use crate::router::route_request;
use crate::shared::SharedState;

pub struct HttpServer {
    listener: TcpListener,
    shared_state: SharedState,
}

impl HttpServer {
    pub async fn new(
        addr: &str,
        shared_state: SharedState,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(HttpServer {
            listener,
            shared_state,
        })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            println!("New connection from: {}", addr);

            // Spawn a new task for each connection
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).await?;

        let request_str = str::from_utf8(&buffer[..bytes_read])?;
        println!("Received request:\n{}", request_str);

        let http_request = HttpRequest::parse(request_str)?;

        let response = route_request(&http_request).await;

        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;

        Ok(())
    }
}
