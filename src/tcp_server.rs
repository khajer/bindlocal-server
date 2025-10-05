use crate::shared::SharedState;
use std::collections::HashMap;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc;

use crate::shared::TicketRequestHttp;

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
        let (tx_tcp, mut rx_tcp) = mpsc::unbounded_channel::<TicketRequestHttp>();
        shared_state
            .register_tcp_client(client_id.to_string(), tx_tcp)
            .await;

        // Send welcome message
        let welcome = format!(
            "Connected the Server\nhost: http://{}.localhost:8080",
            client_id
        );
        stream.write_all(welcome.as_bytes()).await?;

        loop {
            select! {
                msg = rx_tcp.recv() => {
                    match msg {
                        Some(ticket) => {

                            let message = ticket.data;
                            if let Err(e) = stream.write_all(&message).await {
                                eprintln!("Error sending direct message to TCP client {}: {}", client_id, e);
                                shared_state.send_to_http_client(client_id.as_str(), vec![]).await;
                                break;
                            }
                            if let Err(e) = stream.flush().await {
                                eprintln!("Error flushing TCP stream: {}", e);
                                shared_state.send_to_http_client(client_id.as_str(), vec![]).await;
                                break;
                            }

                            let mut buffer = Vec::new();
                            let mut tmp = [0u8; 1024];
                            let header_end;
                            loop {
                                let n = stream.read(&mut tmp).await?;
                                if n == 0 {
                                    return Err("connection closed before headers".into());
                                }
                                buffer.extend_from_slice(&tmp[..n]);
                                if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                                    header_end = pos + 4;
                                    break;
                                }
                            }
                            let header_text = String::from_utf8_lossy(&buffer[..header_end]);
                            let mut headers = HashMap::new();
                            for line in header_text.lines().skip(1) {
                                if let Some((k, v)) = line.split_once(": ") {
                                    headers.insert(k.to_string(), v.to_string());
                                }
                            }

                            if let Some(len) = headers.get("Content-Length") {
                                // println!("response case: Content-Length");
                                let len = len.parse::<usize>()?;
                                while buffer.len() < header_end + len {
                                    let n = stream.read(&mut tmp).await?;
                                    if n == 0 {
                                        break;
                                    }
                                    buffer.extend_from_slice(&tmp[..n]);
                                }
                            } else if headers
                                .get("Transfer-Encoding")
                                .map(|v| v.to_ascii_lowercase())
                                == Some("chunked".into())
                            {
                                loop {
                                    if buffer[header_end..].windows(5).any(|w| w == b"0\r\n\r\n") {
                                        break;
                                    }
                                    // Read more data
                                    let n = stream.read(&mut tmp).await?;
                                    if n == 0 {
                                        return Err("connection closed before chunked terminator".into());
                                    }
                                    buffer.extend_from_slice(&tmp[..n]);
                                }

                                if let Some(terminator_pos) = buffer[header_end..]
                                    .windows(5)
                                    .position(|w| w == b"0\r\n\r\n")
                                {
                                    let end_pos = header_end + terminator_pos + 5; // Include the terminator
                                    buffer.truncate(end_pos);
                                }
                            } else {
                                // case return only header for example: 304, 201
                            }
                            println!("Receive from client: {} bytes", buffer.len());
                            shared_state.send_to_http_client(ticket.name.as_str(), buffer).await;
                        },
                        None => {
                            println!("TCP client application close: [{}] ", client_id);
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
