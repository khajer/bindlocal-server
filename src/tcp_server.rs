use crate::shared::SharedState;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc;

pub struct TcpServer {
    listener: TcpListener,
    shared_state: SharedState,
}

impl TcpServer {
    pub async fn new(
        addr: &str,
        shared_state: SharedState,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(TcpServer {
            listener,
            shared_state,
        })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        println!("TCP Server listening for raw TCP connections...");
        let mut client_cnt = 1;
        loop {
            let (socket, addr) = self.listener.accept().await?;
            println!("New TCP connection from: {}", addr);

            let client_id = format!("{:04}", client_cnt);
            client_cnt += 1;
            println!("client id [{}]", client_id);

            let shared_state = self.shared_state.clone();

            // Spawn a new task for each TCP connection
            tokio::spawn(async move {
                if let Err(e) =
                    Self::handle_tcp_connection(socket, shared_state, client_id.to_string()).await
                {
                    eprintln!("Error handling TCP connection: {}", e);
                }
            });
        }
    }

    async fn handle_tcp_connection(
        mut stream: TcpStream,
        shared_state: SharedState,
        client_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (tx_tcp, mut rx_tcp) = mpsc::unbounded_channel::<String>();

        shared_state
            .register_tcp_client(client_id.to_string(), tx_tcp)
            .await;

        // Send welcome message
        let welcome = "Welcome to Raw TCP Server!\nCommands: echo <msg>, time, status, quit\n> ";
        stream.write_all(welcome.as_bytes()).await?;

        let mut buffer = [0; 1024];

        loop {
            select! {
                // Handle direct messages sent to this specific client
                msg = rx_tcp.recv() => {
                    match msg {
                        Some(message) => {

                            if let Err(e) = stream.write_all(message.as_bytes()).await {
                                eprintln!("Error sending direct message to TCP client {}: {}", client_id, e);
                                break;
                            }
                            if let Err(e) = stream.flush().await {
                                eprintln!("Error flushing TCP stream: {}", e);
                                break;
                            }

                            let result = stream.read(&mut buffer).await?;
                            let rec_msg = str::from_utf8(&buffer[..result])?.trim();
                            println!("TCP received from {}: {}", client_id, rec_msg);

                            shared_state.send_to_http_client("0001", rec_msg).await;

                        },
                        None => {
                            println!("TCP client {} channel closed", client_id);
                            break;
                        }
                    }
                },
            }
        }
        Ok(())
    }
}
