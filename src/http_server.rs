use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::response::HttpResponse;
use crate::shared::{Message, SharedState};

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

        // waiting for
        let (tx_http, mut rx_http) = mpsc::unbounded_channel::<String>();

        let client_id = "0001";
        shared_state
            .register_http_client(client_id.to_string(), tx_http)
            .await;

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

        if !shared_state.send_to_tcp_client("0001", "PING").await {
            println!("sending fails");
        }

        let rec = rx_http.recv().await.unwrap();
        println!("receive , {}", rec);

        // let receive = rx_http.recv();
        // match receive {
        //     Some(message) => {
        //         println!("test");
        //     }
        //     _ => {
        //         println!("error");
        //     }
        // }
        // let http_request = HttpRequest::parse(request_str)?;

        // let response = route_request(&http_request).await;
        let response = HttpResponse::ok_text(rec.as_str()).to_string();
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;

        Ok(())
    }
}
