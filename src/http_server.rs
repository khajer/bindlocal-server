use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedReceiver;
// use chrono::{Datelike, Local, Timelike};
use rand::Rng;

use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::shared::SharedState;
use crate::shared::TicketRequestHttp;

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
            let (socket, _addr) = self.listener.accept().await?;

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
        loop {
            let mut buf = vec![0u8; 1024]; // Initial capacity
            let mut total_data = Vec::new();
            loop {
                let n = stream.read(&mut buf).await?;
                if n == 0 {
                    break; // EOF
                }
                total_data.extend_from_slice(&buf[..n]);

                if total_data.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }

            let header = total_data.windows(4).position(|w| w == b"\r\n\r\n");
            let headers_end = match header {
                Some(value) => value + 4,
                None => {
                    break;
                }
            };

            let headers_str = str::from_utf8(&total_data[..headers_end - 4])?.to_string();
            let content_length = HttpRequest::parse_content_length(headers_str.clone());
            // let connection_type = HttpRequest::parse_connection(headers_str.clone());

            if let Some(body_length) = content_length {
                let body_data_received = total_data.len() - headers_end;
                let remaining_body = body_length - body_data_received;
                if remaining_body > 0 {
                    let mut body_buf = vec![0u8; remaining_body];
                    let mut bytes_read = 0;

                    while bytes_read < remaining_body {
                        let n = stream.read(&mut body_buf[bytes_read..]).await?;
                        if n == 0 {
                            return Err("Unexpected EOF while reading body".into());
                        }
                        bytes_read += n;
                    }

                    total_data.extend_from_slice(&body_buf);
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

            let trx_id = generate_trx_id();

            let ticket = TicketRequestHttp {
                name: format!("{trx_id}"),
                data: total_data,
            };

            let (tx_http, rx_http) = mpsc::unbounded_channel::<Vec<u8>>();

            shared_state
                .register_http_client(ticket.name.clone(), tx_http)
                .await;

            if !shared_state
                .send_to_tcp_client(client_id.as_str(), ticket)
                .await
            {
                return Err("sending fails".into());
            }

            // waiting for response from TCP client
            wait_for_tcp_response(rx_http, &mut stream).await;

            if let Some(conn_type) = HttpRequest::parse_connection(headers_str.clone()) {
                if conn_type == "close" {
                    break;
                }
            }
        }
        Ok(())
    }
}
async fn wait_for_tcp_response(mut rx_http: UnboundedReceiver<Vec<u8>>, stream: &mut TcpStream) {
    match rx_http.recv().await {
        Some(value) => {
            if !value.is_empty() {
                stream.write_all(&value).await.unwrap();
            } else {
                let response = HttpResponse::not_found().to_string();
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        }
        None => {
            let response = HttpResponse::service_unavailable().to_string();
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    }
    stream.flush().await.unwrap();
}

fn generate_trx_id() -> String {
    let mut rng = rand::rng();
    let tx_id: u32 = rng.random_range(10000000..100000000); // 8-digit number
    format!("tx-{:x}", tx_id)
}
