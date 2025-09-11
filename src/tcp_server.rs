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
        let welcome = format!(
            "Connected the Server\nhost: http://{}.localhost:8080",
            client_id
        );
        stream.write_all(welcome.as_bytes()).await?;

        // let mut buffer = [0; 4096];

        loop {
            select! {
                // Handle direct messages sent to this specific client
                msg = rx_tcp.recv() => {
                    match msg {
                        Some(message) => {
                            if let Err(e) = stream.write(message.as_bytes()).await {
                                eprintln!("Error sending direct message to TCP client {}: {}", client_id, e);
                                shared_state.send_to_http_client(client_id.as_str(), vec![]).await;
                                break;
                            }
                            if let Err(e) = stream.flush().await {
                                eprintln!("Error flushing TCP stream: {}", e);
                                shared_state.send_to_http_client(client_id.as_str(), vec![]).await;
                                break;
                            }


                            // wait and receive message.

                            let mut buf = vec![0u8; 4096]; // Initial capacity
                            let mut total_data = Vec::new();
                            loop {
                                println!("start");
                                let n = stream.read(&mut buf).await?;
                                println!("read");
                                if n == 0 {
                                    break; // EOF
                                }
                                println!("loop");
                                total_data.extend_from_slice(&buf[..n]);
                                println!("extend");

                                if total_data.windows(4).any(|w| w == b"\r\n\r\n") {
                                           println!("Complete HTTP headers received");
                                           break;
                                }
                            }
                            println!("Response received, length: {} bytes", total_data.len());
                            // println!("TCP received from {}: {}", client_id, total_data.len());
                            // let rec_msg = str::from_utf8(&total_data)?.trim();
                            // Vec<u8>
                            shared_state.send_to_http_client(client_id.as_str(), total_data).await;

                            // let result = stream.read(&mut buffer).await?;
                            // let rec_msg = str::from_utf8(&buffer[..result])?.trim();
                            // println!("TCP received from {}: {}", client_id, rec_msg);
                            // shared_state.send_to_http_client(client_id.as_str(), rec_msg).await;


                            // let mut full_buffer = Vec::new();
                            // println!("**** before read");
                            // let result = stream(&mut full_buffer).await?;
                            // println!("**** Read completed");
                            // let rec_msg = str::from_utf8(&full_buffer[..result])?.trim();
                            // println!("**** Read completed\n {:?}", rec_msg);
                            // shared_state.send_to_http_client(client_id.as_str(), rec_msg).await;

                        },
                        None => {
                            println!("TCP client {} channel closed", client_id);
                            shared_state
                                .unregister_tcp_client(client_id.as_str()).await;
                            break;
                        }
                    }
                },
            }
        }
        Ok(())
    }
}
