use crate::shared::SharedState;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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

        loop {
            let (socket, addr) = self.listener.accept().await?;
            println!("New TCP connection from: {}", addr);

            // Spawn a new task for each TCP connection
            tokio::spawn(async move {
                if let Err(e) = Self::handle_tcp_connection(socket).await {
                    eprintln!("Error handling TCP connection: {}", e);
                }
            });
        }
    }

    async fn handle_tcp_connection(
        mut stream: TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Send welcome message
        let welcome = "Welcome to Raw TCP Server!\nCommands: echo <msg>, time, status, quit\n> ";
        stream.write_all(welcome.as_bytes()).await?;

        let mut buffer = [0; 1024];

        loop {
            let bytes_read = stream.read(&mut buffer).await?;

            if bytes_read == 0 {
                println!("TCP client disconnected");
                break;
            }

            let message = str::from_utf8(&buffer[..bytes_read])?.trim();
            println!("TCP received: {}", message);

            let response = Self::process_tcp_command(message).await;

            if response == "QUIT" {
                let goodbye = "Goodbye!\n";
                stream.write_all(goodbye.as_bytes()).await?;
                break;
            }

            let full_response = format!("{}\n> ", response);
            stream.write_all(full_response.as_bytes()).await?;
            stream.flush().await?;
        }

        Ok(())
    }

    async fn process_tcp_command(command: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return "Invalid command. Try: echo <message>, time, status, or quit".to_string();
        }

        match parts[0].to_lowercase().as_str() {
            "echo" => {
                if parts.len() > 1 {
                    format!("Echo: {}", parts[1..].join(" "))
                } else {
                    "Echo: (no message provided)".to_string()
                }
            },
            "time" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                format!("Current timestamp: {}", timestamp)
            },
            "status" => {
                "TCP Server Status: Running | Type 'quit' to disconnect".to_string()
            },
            "quit" | "exit" => "QUIT".to_string(),
            "help" => {
                "Available commands:\n  echo <message> - Echo back your message\n  time - Get current timestamp\n  status - Server status\n  quit - Disconnect".to_string()
            },
            _ => {
                format!("Unknown command: '{}'. Try: echo, time, status, help, or quit", parts[0])
            }
        }
    }
}
