use crate::shared::SharedState;
use std::collections::HashMap;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::Instant;

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
        let (tx_tcp, mut rx_tcp) = mpsc::unbounded_channel::<Vec<u8>>();

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


                            // let mut response_data: Vec<u8> = Vec::new();
                            // if let Err(e) = stream_local.read_to_end(&mut response_data).await {
                            //     eprintln!("Error flushing TCP stream: {}", e);
                            // }
                            // println!("response completed.");
                            // ---
                            let mut buf = vec![0u8; 10240*4]; // Initial capacity
                            let mut total_data = Vec::new();
                            loop {

                                let n = stream.read(&mut buf).await?; // read from tcp stream
                                if n == 0 {
                                    break; // EOF
                                }
                                println!("n {n}");
                                total_data.extend_from_slice(&buf[..n]);

                                if total_data.windows(4).any(|w| w == b"\r\n\r\n") {
                                           println!("received completed ");
                                           break;
                                }
                            }
                            println!("recieved size: {} bytes", total_data.len());

                            // tokio::fs::write("./tmp/response_raw.html", &total_data).await?;

                            shared_state.send_to_http_client(client_id.as_str(), total_data).await;

                            // ----

                            // println!(">>> receiving process");
                            // let mut buffer = Vec::new();
                            // let mut tmp = [0u8; 1024];
                            // let header_end;
                            // let time = Instant::now();
                            // loop {
                            //     let n = stream.read(&mut tmp).await?;
                            //     if n == 0 {
                            //         return Err("connection closed before headers".into());
                            //     }
                            //     buffer.extend_from_slice(&tmp[..n]);
                            //     if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                            //         header_end = pos + 4;
                            //         break;
                            //     }
                            // }
                            // // --- Parse headers (just enough to know how much to read) ---
                            // let header_text = String::from_utf8_lossy(&buffer[..header_end]);
                            // let mut headers = HashMap::new();
                            // for line in header_text.lines().skip(1) {
                            //     if let Some((k, v)) = line.split_once(": ") {
                            //         headers.insert(k.to_string(), v.to_string());
                            //     }
                            // }
                            // // --- Parse headers (just enough to know how much to read) ---
                            // let header_text = String::from_utf8_lossy(&buffer[..header_end]);
                            // let mut headers = HashMap::new();
                            // for line in header_text.lines().skip(1) {
                            //     if let Some((k, v)) = line.split_once(": ") {
                            //         headers.insert(k.to_string(), v.to_string());
                            //     }
                            // }

                            // // --- Read the body depending on headers ---
                            // if let Some(len) = headers.get("Content-Length") {
                            //     let len = len.parse::<usize>()?;
                            //     while buffer.len() < header_end + len {
                            //         let n = stream.read(&mut tmp).await?;
                            //         if n == 0 {
                            //             break;
                            //         }
                            //         buffer.extend_from_slice(&tmp[..n]);
                            //     }
                            // } else if headers
                            //     .get("Transfer-Encoding")
                            //     .map(|v| v.to_ascii_lowercase())
                            //     == Some("chunked".into())
                            // {
                            //     let mut rest = buffer[header_end..].to_vec();
                            //     loop {
                            //         // Ensure we have a full line
                            //         while !rest.windows(2).any(|w| w == b"\r\n") {
                            //             let n = stream.read(&mut tmp).await?;
                            //             if n == 0 {
                            //                 return Err("connection closed during chunk size".into());
                            //             }
                            //             rest.extend_from_slice(&tmp[..n]);
                            //         }

                            //         // Get chunk size
                            //         let pos = rest.windows(2).position(|w| w == b"\r\n").unwrap();
                            //         let line = String::from_utf8_lossy(&rest[..pos]);
                            //         let size = usize::from_str_radix(line.trim(), 16)?;
                            //         let chunk_header_len = pos + 2;

                            //         // Copy chunk header into buffer
                            //         buffer.extend_from_slice(&rest[..chunk_header_len]);
                            //         rest.drain(..chunk_header_len);

                            //         if size == 0 {
                            //             buffer.extend_from_slice(b"\r\n"); // final CRLF
                            //             break;
                            //         }

                            //         // Ensure we have full chunk
                            //         while rest.len() < size + 2 {
                            //             let n = stream.read(&mut tmp).await?;
                            //             if n == 0 {
                            //                 return Err("connection closed during chunk body".into());
                            //             }
                            //             rest.extend_from_slice(&tmp[..n]);
                            //         }

                            //         // Copy chunk data + CRLF into buffer
                            //         buffer.extend_from_slice(&rest[..size + 2]);
                            //         rest.drain(..size + 2);
                            //     }
                            // } else {
                            //     // Fallback: read until connection closes
                            //     loop {
                            //         let n = stream.read(&mut tmp).await?;
                            //         if n == 0 {
                            //             break;
                            //         }
                            //         buffer.extend_from_slice(&tmp[..n]);
                            //     }
                            // }
                            // println!(
                            //     "[+chunk] size: {} bytes, elapsed: {:?}",
                            //     buffer.len(),
                            //     time.elapsed()
                            // );

                            // shared_state.send_to_http_client(client_id.as_str(), buffer).await;

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

    async fn read_data_stream(
        mut stream: TcpStream,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        let mut tmp = [0u8; 1024];
        let header_end;
        let time = Instant::now();
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
        // --- Parse headers (just enough to know how much to read) ---
        let header_text = String::from_utf8_lossy(&buffer[..header_end]);
        let mut headers = HashMap::new();
        for line in header_text.lines().skip(1) {
            if let Some((k, v)) = line.split_once(": ") {
                headers.insert(k.to_string(), v.to_string());
            }
        }
        // --- Parse headers (just enough to know how much to read) ---
        let header_text = String::from_utf8_lossy(&buffer[..header_end]);
        let mut headers = HashMap::new();
        for line in header_text.lines().skip(1) {
            if let Some((k, v)) = line.split_once(": ") {
                headers.insert(k.to_string(), v.to_string());
            }
        }

        // --- Read the body depending on headers ---
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
            let mut rest = buffer[header_end..].to_vec();
            loop {
                // Ensure we have a full line
                while !rest.windows(2).any(|w| w == b"\r\n") {
                    let n = stream.read(&mut tmp).await?;
                    if n == 0 {
                        return Err("connection closed during chunk size".into());
                    }
                    rest.extend_from_slice(&tmp[..n]);
                }

                // Get chunk size
                let pos = rest.windows(2).position(|w| w == b"\r\n").unwrap();
                let line = String::from_utf8_lossy(&rest[..pos]);
                let size = usize::from_str_radix(line.trim(), 16)?;
                let chunk_header_len = pos + 2;

                // Copy chunk header into buffer
                buffer.extend_from_slice(&rest[..chunk_header_len]);
                rest.drain(..chunk_header_len);

                if size == 0 {
                    buffer.extend_from_slice(b"\r\n"); // final CRLF
                    break;
                }

                // Ensure we have full chunk
                while rest.len() < size + 2 {
                    let n = stream.read(&mut tmp).await?;
                    if n == 0 {
                        return Err("connection closed during chunk body".into());
                    }
                    rest.extend_from_slice(&tmp[..n]);
                }

                // Copy chunk data + CRLF into buffer
                buffer.extend_from_slice(&rest[..size + 2]);
                rest.drain(..size + 2);
            }
        } else {
            // Fallback: read until connection closes
            loop {
                let n = stream.read(&mut tmp).await?;
                if n == 0 {
                    break;
                }
                buffer.extend_from_slice(&tmp[..n]);
            }
        }
        println!(
            "[+chunk] size: {} bytes, elapsed: {:?}",
            buffer.len(),
            time.elapsed()
        );

        Ok(buffer)
    }
}
