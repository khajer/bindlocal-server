use axum::{
    Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::{RwLock, broadcast};

// Query parameters for the GET endpoint
#[derive(Debug, Deserialize)]
pub struct SendMessage {
    pub msg: String,
}

// WebSocket client info
#[derive(Debug, Clone)]
struct Client {
    id: u64,
    addr: SocketAddr,
}

// Shared state
#[derive(Debug)]
pub struct AppState {
    clients: RwLock<HashMap<u64, Client>>,
    tx: broadcast::Sender<String>, // Simple string broadcast
    next_id: AtomicU64,
}

impl AppState {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            clients: RwLock::new(HashMap::new()),
            tx,
            next_id: AtomicU64::new(1),
        }
    }

    async fn add_client(&self, addr: SocketAddr) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let client = Client { id, addr };
        self.clients.write().await.insert(id, client);
        println!("✅ Client {} connected", id);
        id
    }

    async fn remove_client(&self, id: u64) {
        self.clients.write().await.remove(&id);
        println!("🔌 Client {} disconnected", id);
    }

    async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    // Broadcast message to all WebSocket clients
    fn broadcast(&self, message: String) -> Result<usize, broadcast::error::SendError<String>> {
        self.tx.send(message)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/send", get(send_message_handler)) // GET /send?msg=aaa
        .with_state(app_state);

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("🚀 Simple GET to WebSocket Broadcast Server");
    println!("📡 Server: http://{}", addr);
    println!("🔌 WebSocket: ws://{}/ws", addr);
    println!("📤 Send message: GET http://{}/send?msg=your_message", addr);
    println!();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state, addr))
}

async fn handle_websocket(socket: WebSocket, state: Arc<AppState>, addr: SocketAddr) {
    let client_id = state.add_client(addr).await;
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Send welcome message
    let welcome = format!(
        "Welcome! Client ID: {}. Connected clients: {}",
        client_id,
        state.client_count().await
    );
    let _ = sender.send(Message::Text(welcome)).await;

    // Task to handle broadcasts from GET endpoint
    let broadcast_task = tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            // Send the broadcasted message to this WebSocket client
            if sender.send(Message::Text(message)).await.is_err() {
                break;
            }
        }
    });

    // Task to handle incoming WebSocket messages (optional)
    let message_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                println!("📨 Client {} sent: {}", client_id, text);
                // You could broadcast WebSocket messages too if needed
                // let _ = state.broadcast(format!("Client {}: {}", client_id, text));
            } else if let Ok(Message::Close(_)) = msg {
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = broadcast_task => {},
        _ = message_task => {},
    }

    // Cleanup when client disconnects
    state.remove_client(client_id).await;
}

// GET endpoint: /send?msg=aaa
async fn send_message_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SendMessage>,
) -> impl IntoResponse {
    let message = params.msg;
    println!("📤 GET /send called with message: '{}'", message);

    // Broadcast the message to all WebSocket clients
    match state.broadcast(message.clone()) {
        Ok(receiver_count) => {
            println!(
                "✅ Message '{}' broadcasted to {} clients",
                message, receiver_count
            );

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": message,
                    "broadcasted_to": receiver_count,
                    "connected_clients": state.client_count().await
                })),
            )
        }
        Err(e) => {
            println!("❌ Failed to broadcast message: {}", e);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Failed to broadcast message",
                    "connected_clients": state.client_count().await
                })),
            )
        }
    }
}
