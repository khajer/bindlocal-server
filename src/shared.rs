use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

pub struct TicketRequestHttp {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct SharedState {
    pub tcp_connections: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<TicketRequestHttp>>>>,
    pub http_connections: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Vec<u8>>>>>,
    // pub rx_http_clients: Arc<Mutex<HashMap<String, mpsc::UnboundedReceiver<Vec<u8>>>>>,
}

impl SharedState {
    pub fn new() -> Self {
        let state = SharedState {
            tcp_connections: Arc::new(Mutex::new(HashMap::new())),
            http_connections: Arc::new(Mutex::new(HashMap::new())),
            // rx_http_clients: Arc::new(Mutex::new(HashMap::new())),
        };
        state
    }

    pub async fn send_to_tcp_client(&self, client_id: &str, ticket: TicketRequestHttp) -> bool {
        let connections = self.tcp_connections.lock().await;
        if let Some(tx_tcp) = connections.get(client_id) {
            tx_tcp.send(ticket).is_ok()
        } else {
            false
        }
    }

    pub async fn send_to_http_client(&self, client_id: &str, message: Vec<u8>) -> bool {
        let connections = self.http_connections.lock().await;
        if let Some(tx_http) = connections.get(client_id) {
            match tx_http.send(message) {
                Ok(_) => {
                    println!("send message to HTTP client");
                    true
                }
                Err(e) => {
                    eprintln!("Failed to send message to HTTP client: {}", e);
                    false
                }
            }
        } else {
            println!("cannot connect client id {}", client_id);
            false
        }
    }

    pub async fn register_tcp_client(
        &self,
        client_id: String,
        tx: mpsc::UnboundedSender<TicketRequestHttp>,
    ) {
        let mut connections = self.tcp_connections.lock().await;
        connections.insert(client_id, tx);
    }
    pub async fn unregister_tcp_client(&self, client_id: &str) {
        let mut connections = self.tcp_connections.lock().await;
        connections.remove(client_id);
    }

    pub async fn register_http_client(
        &self,
        client_id: String,
        tx: mpsc::UnboundedSender<Vec<u8>>,
    ) {
        let mut connections = self.http_connections.lock().await;

        connections.insert(client_id, tx);
    }

    // rx for front
    // pub async fn register_rx_http_client(
    //     &self,
    //     client_id: &str,
    //     mut rx: UnboundedReceiver<Vec<u8>>,
    // ) {
    //     let mut rx_http_clients = self.rx_http_clients.lock().await;
    //     rx_http_clients.insert(client_id, rx);
    // }
    // pub async fn get_rx_http_client(
    //     &self,
    //     client_id: &str,
    // ) -> Option<mpsc::UnboundedReceiver<Vec<u8>>> {
    //     let rx_http_clients = self.rx_http_clients.lock().await;
    //     let connections = self.tcp_connections.lock().await;

    //     rx_http_clients.get(client_id)
    // }
    //
}
