use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub from: String, // "http" or "tcp"
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct SharedState {
    // Channel for broadcasting messages to all TCP clients
    pub broadcast_tx: broadcast::Sender<Message>,

    // Channel for sending messages from HTTP to TCP
    pub http_to_tcp_tx: mpsc::UnboundedSender<Message>,

    // Storage for messages
    pub messages: Arc<Mutex<Vec<Message>>>,

    // Active TCP connections
    pub tcp_connections: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,

    // Active HTTP conections
    pub http_connections: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
}

impl SharedState {
    pub fn new() -> (
        Self,
        broadcast::Receiver<Message>,
        mpsc::UnboundedReceiver<Message>,
    ) {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(100);
        let (http_to_tcp_tx, http_to_tcp_rx) = mpsc::unbounded_channel();

        let state = SharedState {
            broadcast_tx,
            http_to_tcp_tx,
            messages: Arc::new(Mutex::new(Vec::new())),
            tcp_connections: Arc::new(Mutex::new(HashMap::new())),
            http_connections: Arc::new(Mutex::new(HashMap::new())),
        };

        (state, broadcast_rx, http_to_tcp_rx)
    }

    // pub async fn add_message(&self, message: Message) {
    //     println!("add_message: {:?}", message);
    //     let mut messages = self.messages.lock().await;
    //     messages.push(message.clone());

    //     // Keep only last 100 messages
    //     if messages.len() > 100 {
    //         messages.remove(0);
    //     }

    //     // Broadcast to all subscribers
    //     let _ = self.broadcast_tx.send(message);
    // }

    // pub async fn get_messages(&self) -> Vec<Message> {
    //     self.messages.lock().await.clone()
    // }

    pub async fn send_to_tcp_client(&self, client_id: &str, message: &str) -> bool {
        let connections = self.tcp_connections.lock().await;

        if let Some(tx_tcp) = connections.get(client_id) {
            tx_tcp.send(message.to_string()).is_ok()
        } else {
            false
        }
    }

    pub async fn send_to_http_client(&self, client_id: &str, message: &str) -> bool {
        let connections = self.http_connections.lock().await;

        if let Some(tx_http) = connections.get(client_id) {
            tx_http.send(message.to_string()).is_ok()
        } else {
            false
        }
    }

    pub async fn register_tcp_client(&self, client_id: String, tx: mpsc::UnboundedSender<String>) {
        let mut connections = self.tcp_connections.lock().await;
        connections.insert(client_id, tx);
    }

    pub async fn register_http_client(&self, client_id: String, tx: mpsc::UnboundedSender<String>) {
        let mut connections = self.http_connections.lock().await;
        connections.insert(client_id, tx);
    }

    // pub async fn unregister_tcp_client(&self, client_id: &str) {
    //     let mut connections = self.tcp_connections.lock().await;
    //     connections.remove(client_id);
    // }
}
