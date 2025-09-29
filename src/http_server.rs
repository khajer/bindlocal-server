use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use chrono::{Datelike, Local, Timelike};
// use tokio::fs::File;
use uuid::Uuid;

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

        let headers_end = total_data
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .unwrap()
            + 4;
        let headers_str = str::from_utf8(&total_data[..headers_end - 4])?;
        let content_length = HttpRequest::parse_content_length(headers_str);

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

        let trx_id = Uuid::new_v4();
        save_log_req_resp(format!("[{trx_id}] request").as_str(), &total_data).await;

        // waiting for
        let (tx_http, mut rx_http) = mpsc::unbounded_channel::<Vec<u8>>();

        shared_state
            .register_http_client(client_id.to_string(), tx_http)
            .await;

        if !shared_state
            .send_to_tcp_client(client_id.as_str(), total_data)
            .await
        {
            return Err("sending fails".into());
        }

        // waiting for response from TCP client
        match rx_http.recv().await {
            Some(value) => {
                if !value.is_empty() {
                    save_log_req_resp(format!("[{trx_id}] response").as_str(), &value).await;
                    stream.write_all(&value).await?;
                    // println!("Receive from client: {} bytes", value.len());
                } else {
                    println!("[{trx_id}] response: Empty");
                    shared_state.unregister_tcp_client(client_id.as_str()).await;
                    let response = HttpResponse::not_found().to_string();
                    stream.write_all(response.as_bytes()).await?;
                }
            }
            None => {
                println!("[{trx_id}] response: None");
                let response = HttpResponse::server_response_error().to_string();
                stream.write_all(response.as_bytes()).await?;
            }
        }
        stream.flush().await?;
        Ok(())
    }
}
async fn save_log_req_resp(intro_str: &str, data: &[u8]) {
    let now = Local::now();

    let intro_str = format!(
        "[{}{:02}{:02} {:02}:{:02}.{:02}] {intro_str} \n",
        now.year(),
        now.month().to_string(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );

    println!("{intro_str}");
    println!("{}", String::from_utf8_lossy(data));

    // let filename = format!("logs/{}{}{}.log", now.year(), now.month(), now.day());
    // let mut f = File::options()
    //     .append(true)
    //     .create(true)
    //     .open(filename)
    //     .await
    //     .unwrap();
    // f.write_all(intro_str.as_bytes()).await.unwrap();
    // f.write_all(&data).await.unwrap();
}
