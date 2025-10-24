use crate::shared::SharedState;
use rand::Rng;
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

const MINIMUM_CLIENT_VERSION: &str = "0.0.2";

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
        loop {
            let (socket, addr) = self.listener.accept().await?;
            tracing::info!("New TCP connection from: {}", addr);

            let shared_state = self.shared_state.clone();

            // Spawn a new task for each TCP connection
            tokio::spawn(async move {
                if let Err(e) = Self::handle_tcp_connection(socket, shared_state).await {
                    eprintln!("Error handling TCP connection: {}", e);
                }
            });
        }
    }

    async fn handle_tcp_connection(
        mut stream: TcpStream,
        mut shared_state: SharedState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // check version available
        let mut first_access = [0u8; 1024];
        let n = stream.read(&mut first_access).await?;
        if n == 0 {
            return Ok(());
        }

        let incoming_message = String::from_utf8_lossy(&first_access[..n]);
        if let Some(version) = incoming_message.split(" ").nth(1) {
            if !check_available_version(&version, MINIMUM_CLIENT_VERSION) {
                let txt_resp = "ERR001:request_higher_version";
                stream.write_all(txt_resp.as_bytes()).await?;
                return Ok(());
            }
        }
        let mut client_id;
        if let Some(sub_domain_name) = incoming_message.split(" ").nth(2) {
            client_id = sub_domain_name.to_string();
            let mut cnt = 1;
            while shared_state.check_duplicate_subdomain(client_id.clone()) {
                client_id = format!("{}-{}", client_id, cnt);
                cnt += 1;
            }
        } else {
            client_id = generate_name();
            while shared_state.check_duplicate_subdomain(client_id.clone()) {
                client_id = generate_name();
            }
        }
        tracing::info!("client id [{}]", client_id);
        let (tx_tcp, mut rx_tcp) = mpsc::unbounded_channel::<TicketRequestHttp>();
        shared_state
            .register_tcp_client(client_id.to_string(), tx_tcp)
            .await;

        // Send welcome message
        let welcome = format!("{}", client_id);
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
                                    return Err("Connection closed before headers".into());
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
                                        return Err("Connection closed before chunked terminator".into());
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
                            shared_state.send_to_http_client(ticket.name.as_str(), buffer).await;
                        },
                        None => {
                            tracing::info!("TCP client application close: [{}] ", client_id);
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

fn generate_name() -> String {
    let mut rng = rand::rng();
    let name = format!("app-{:04}", rng.random_range(0..10000));
    name
}

fn parse_version(version_str: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    match (parts[0].parse(), parts[1].parse(), parts[2].parse()) {
        (Ok(major), Ok(minor), Ok(patch)) => Some((major, minor, patch)),
        _ => None,
    }
}

/// Checks if the `first_access` version is greater than or equal to the `current_version`.
fn check_available_version(first_access: &str, current_version: &str) -> bool {
    let first_parsed = match parse_version(first_access) {
        Some(v) => v,
        None => return false,
    };

    let current_parsed = match parse_version(current_version) {
        Some(v) => v,
        None => return false,
    };
    first_parsed >= current_parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_name() {
        let result = generate_name();
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_check_available_version_002() {
        let first_access = "0.0.2";
        assert_eq!(check_available_version(first_access, "0.0.2"), true);
    }
    #[test]
    fn test_check_available_version_001() {
        let first_access = "0.0.1";
        assert_eq!(check_available_version(first_access, "0.0.2"), false);
    }
    #[test]
    fn test_check_available_version_003() {
        let first_access = "0.0.3";
        assert_eq!(check_available_version(first_access, "0.0.2"), true);
    }
    #[test]
    fn test_check_available_version_random_text() {
        let first_access = "fdsfdsfjds9]nsfdlksjdfl";
        assert_eq!(check_available_version(first_access, "0.0.2"), false);
    }

    #[test]
    fn test_parse_version() {
        let version_str = "1.2.3";
        let expected = Some((1, 2, 3));
        assert_eq!(parse_version(version_str), expected);
    }
}
