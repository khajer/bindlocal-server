use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
#[derive(Clone)]
pub struct SharedState {
    pub tcp_connections: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
    pub http_connections: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
}

impl SharedState {
    pub fn new() -> Self {
        let state = SharedState {
            tcp_connections: Arc::new(Mutex::new(HashMap::new())),
            http_connections: Arc::new(Mutex::new(HashMap::new())),
        };
        state
    }

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
    pub async fn unregister_tcp_client(&self, client_id: &str) {
        let mut connections = self.tcp_connections.lock().await;
        connections.remove(client_id);
    }

    pub async fn register_http_client(&self, client_id: String, tx: mpsc::UnboundedSender<String>) {
        let mut connections = self.http_connections.lock().await;

        connections.insert(client_id, tx);
    }
}
