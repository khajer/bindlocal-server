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
                // Handle incoming TCP commands from this client
                // result = stream.read(&mut buffer) => {
                //     match result {
                //         Ok(0) => {
                //             println!("TCP client {} disconnected", client_id);
                //             break;
                //         },
                //         Ok(bytes_read) => {
                //             let message = str::from_utf8(&buffer[..bytes_read])?.trim();
                //             println!("TCP received from {}: {}", client_id, message);

                //             // let response = Self::process_tcp_command(message, &shared_state).await;
                //             let response = Self::process_tcp_command(message).await;

                //             if response == "QUIT" {
                //                 let goodbye = "Goodbye!\n";
                //                 stream.write_all(goodbye.as_bytes()).await?;
                //                 break;
                //             }

                //             let full_response = format!("{}\n> ", response);
                //             stream.write_all(full_response.as_bytes()).await?;
                //             stream.flush().await?;
                //         },
                //         Err(e) => {
                //             eprintln!("Error reading from TCP stream: {}", e);
                //             break;
                //         }
                //     }
                // },

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

                // Handle broadcast messages from HTTP
                // msg = message_receiver.recv() => {
                //     match msg {
                //         Ok(message) => {
                //             let notification = format!("\n[BROADCAST] {}: {}\n> ", message.from, message.content);
                //             if let Err(e) = stream.write_all(notification.as_bytes()).await {
                //                 eprintln!("Error sending broadcast to TCP client {}: {}", client_id, e);
                //                 break;
                //             }
                //             if let Err(e) = stream.flush().await {
                //                 eprintln!("Error flushing TCP stream: {}", e);
                //                 break;
                //             }
                //         },
                //         Err(e) => {
                //             eprintln!("Error receiving broadcast message: {}", e);
                //             // Continue on broadcast errors
                //         }
                //     }
                // }
            }
        }

        Ok(())
    }

    // async fn process_tcp_command(command: &str) -> String {
    //     let parts: Vec<&str> = command.split_whitespace().collect();

    //     if parts.is_empty() {
    //         return "Invalid command. Try: echo <message>, time, status, or quit".to_string();
    //     }

    //     match parts[0].to_lowercase().as_str() {
    //         "echo" => {
    //             if parts.len() > 1 {
    //                 format!("Echo: {}", parts[1..].join(" "))
    //             } else {
    //                 "Echo: (no message provided)".to_string()
    //             }
    //         },
    //         "time" => {
    //             use std::time::{SystemTime, UNIX_EPOCH};
    //             let timestamp = SystemTime::now()
    //                 .duration_since(UNIX_EPOCH)
    //                 .unwrap()
    //                 .as_secs();
    //             format!("Current timestamp: {}", timestamp)
    //         },
    //         "status" => {
    //             "TCP Server Status: Running | Type 'quit' to disconnect".to_string()
    //         },
    //         "quit" | "exit" => "QUIT".to_string(),
    //         "help" => {
    //             "Available commands:\n  echo <message> - Echo back your message\n  time - Get current timestamp\n  status - Server status\n  quit - Disconnect".to_string()
    //         },
    //         _ => {
    //             format!("Unknown command: '{}'. Try: echo, time, status, help, or quit", parts[0])
    //         }
    //     }
    // }
}
