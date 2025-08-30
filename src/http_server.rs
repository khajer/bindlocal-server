use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::request::HttpRequest;
use crate::router::route_request;
use crate::shared::{Message, SharedState};

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
            let shared_state = self.shared_state.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, shared_state).await {
                    eprintln!("Error handling HTTP connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        shared_state: SharedState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).await?;

        let request_str = str::from_utf8(&buffer[..bytes_read])?;
        println!("Received request:\n{}", request_str);

        // let milliseconds_timestamp: u128 = std::time::SystemTime::now()
        //     .duration_since(std::time::UNIX_EPOCH)
        //     .unwrap()
        //     .as_millis();

        // let message = Message {
        //     id: "0001".to_string(),
        //     content: "55555".to_string(),
        //     from: "555".to_string(),
        //     timestamp: milliseconds_timestamp as u64,
        // };

        println!("test send message");
        if shared_state
            .send_to_tcp_client("0001", "test message")
            .await
        {
            println!("send message commpletely");
        } else {
            println!("send message error");
        }

        let http_request = HttpRequest::parse(request_str)?;

        let response = route_request(&http_request).await;

        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;

        Ok(())
    }
}
