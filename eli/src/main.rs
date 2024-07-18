use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket}, Extension, Path},
    routing::get,
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Command {
    account_id: Uuid,
    server_id: Uuid,
    command: String,
    track_id: Option<i32>,
    position: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    server_id: Uuid,
    server_friendly_name: String,
}

struct AppState {
    accounts: Mutex<HashMap<Uuid,AccountState>>,
}

struct AccountState {
    // tx per account
    tx: broadcast::Sender<Command>,
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel(16);
    let state = Arc::new(AppState {
        accounts: Mutex::new(vec![]),
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/command", get(command_handler))
        .layer(Extension(state));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    // Spawn a task to handle messages from the broadcast channel
    tokio::spawn(async move {
        while let Ok(command) = rx.recv().await {
            let msg = serde_json::to_string(&command).unwrap();
            if socket.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming websocket messages (if needed)
    while let Some(Ok(msg)) = socket.recv().await {
        if let axum::extract::ws::Message::Text(data) = msg {
            let command: Command = serde_json::from_str(&data).unwrap();
            // TODO: verify account
            state.accounts.lock().get(&command.account_id).unwrap().tx.send(command).unwrap();
        }
    }
}

async fn command_handler(
    Json(command): Json<Command>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    state.accounts.lock().get(&command.account_id).unwrap().tx.send(command).unwrap();
    StatusCode::OK
}
