use rand::Rng;
use std::str;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedReceiver;

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
            let status_text;
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
            let ip = HttpRequest::parse_x_real_ip(headers_str.clone()).unwrap_or("".to_string());
            let req_txt = HttpRequest::parse_content_request_format(headers_str.clone());
            status_text = format!("{}: {}", ip, &req_txt);

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
            wait_for_tcp_response(rx_http, &mut stream, status_text).await;

            if let Some(conn_type) = HttpRequest::parse_connection(headers_str) {
                if conn_type == "close" {
                    break;
                }
            }
        }
        Ok(())
    }
}
async fn wait_for_tcp_response(
    mut rx_http: UnboundedReceiver<Vec<u8>>,
    stream: &mut TcpStream,
    status_text: String,
) {
    let status_resp: String;
    match rx_http.recv().await {
        Some(value) => {
            if !value.is_empty() {
                let header = value.windows(2).position(|w| w == b"\r\n").unwrap();
                let header_text = String::from_utf8_lossy(&value[0..header]);
                status_resp = parse_response_header(header_text.to_string());
                // stream.write_all(&value).await.unwrap();

                if let Some(v) = check_client_app_error(status_resp.clone()) {
                    stream.write_all(&v).await.unwrap();
                } else {
                    stream.write_all(&value).await.unwrap();
                }
            } else {
                status_resp = "404 Not Found".to_string();
                let response = HttpResponse::not_found().to_string();
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        }
        None => {
            status_resp = "".to_string();
            let response = HttpResponse::service_unavailable().to_string();
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    }
    stream.flush().await.unwrap();
    tracing::info!("{} {}", status_text, status_resp);
}

fn generate_trx_id() -> String {
    let mut rng = rand::rng();
    let tx_id: u32 = rng.random_range(10000000..100000000); // 8-digit number
    format!("tx-{:8}", tx_id)
}

fn parse_response_header(headers: String) -> String {
    if let Some(status_line) = headers.lines().next() {
        if let Some(space_index) = status_line.find(' ') {
            status_line[space_index + 1..].to_string()
        } else {
            status_line.to_string()
        }
    } else {
        String::new()
    }
}

fn check_client_app_error(status_resp: String) -> Option<Vec<u8>> {
    if status_resp.to_lowercase().contains("client_error") {
        let resp_error = HttpResponse::connection_refused().to_string();
        Some(resp_error.as_bytes().to_vec())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_trx_id() {
        let id = generate_trx_id();
        assert_eq!(id.len(), 11); // 8-digit number + "tx-"
    }
    #[test]
    fn test_response_header() {
        let headers = "HTTP/1.1 200 OK".to_string();
        let result = parse_response_header(headers);
        assert_eq!(result, "200 OK");
    }
    #[test]
    fn test_response_header_more_data() {
        let headers = "HTTP/1.1 200 OK\r\n testesttst".to_string();
        let result = parse_response_header(headers);
        assert_eq!(result, "200 OK");
    }

    #[test]
    fn test_check_client_app_error() {
        let error_status = "CLIENT_ERROR:ERR_CONNECTION_REFUSED".to_string();
        let result = check_client_app_error(error_status);
        assert!(result.is_some());
    }
    #[test]
    fn test_check_client_app_error_by_http() {
        let error_status = "HTTP/1.1 500 Internal Server Error".to_string();
        let result = check_client_app_error(error_status);
        assert!(result.is_none());
    }
}
