use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::shared::SharedState;

use tokio::sync::mpsc;

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
            println!("New HTTP connection from: {}", addr);

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
        // wait and receive message.
        let mut buf = vec![0u8; 1024]; // Initial capacity
        let mut total_data = Vec::new();
        loop {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                break; // EOF
            }
            total_data.extend_from_slice(&buf[..n]);

            if total_data.windows(4).any(|w| w == b"\r\n\r\n") {
                println!("Complete HTTP headers received");
                break;
            }
        }

        let response_data: Vec<u8> = total_data.to_vec();

        let request_str = str::from_utf8(&response_data)?;

        let client_id = HttpRequest::get_subdomain(request_str);
        if client_id == "" {
            let response = HttpResponse::not_found().to_string();
            stream.write_all(response.as_bytes()).await?;
            stream.flush().await?;
            return Ok(());
        }

        // waiting for
        let (tx_http, mut rx_http) = mpsc::unbounded_channel::<Vec<u8>>();

        shared_state
            .register_http_client(client_id.to_string(), tx_http)
            .await;

        if !shared_state
            .send_to_tcp_client(client_id.as_str(), response_data)
            .await
        {
            println!("sending fails");
            return Err("sending fails".into());
        }

        let rec = rx_http.recv().await;
        match rec {
            Some(value) => {
                if value.is_empty() {
                    shared_state.unregister_tcp_client(client_id.as_str()).await;
                    let response = HttpResponse::not_found().to_string();
                    stream.write_all(response.as_bytes()).await?;
                } else {
                    stream.write_all(&value).await?;
                }
            }
            None => {
                let response = HttpResponse::not_found().to_string();
                stream.write_all(response.as_bytes()).await?;
            }
        }
        stream.flush().await?;
        Ok(())
    }
}
