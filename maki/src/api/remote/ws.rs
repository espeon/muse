use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::timeout;

use super::hub::Hub;
use super::protocol::{ClientMessage, ErrorCode, Identify};
use crate::api::middleware::jwt::AuthUser;

const IDENTIFY_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    AuthUser { payload }: AuthUser,
    Extension(hub): Extension<Arc<Hub>>,
) -> impl IntoResponse {
    let user_id = match payload.sub.parse::<i32>() {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(error = %e, "remote ws: invalid sub in JWT");
            return (StatusCode::BAD_REQUEST, "invalid user id").into_response();
        }
    };
    ws.on_upgrade(move |socket| handle_socket(socket, user_id, hub))
}

async fn handle_socket(socket: WebSocket, user_id: i32, hub: Arc<Hub>) {
    let (mut sender, mut receiver) = socket.split();

    // Step 1: wait for IDENTIFY (first message).
    let identify = match read_identify(&mut receiver).await {
        Ok(id) => id,
        Err(reason) => {
            let _ = send_error_and_close(&mut sender, ErrorCode::InvalidMessage, reason).await;
            return;
        }
    };

    // Step 2: register the device with the hub. The hub queues a Welcome
    // on the new device's channel; old_sender (if any) is dropped here to
    // close the prior write pump.
    let handle = hub
        .register(
            user_id,
            identify.device_id.clone(),
            identify.name,
            identify.kind,
        )
        .await;
    drop(handle.old_sender);
    let device_id = identify.device_id;
    let cancel = handle.cancel.clone();
    let mut rx = handle.rx;

    // Step 3: write pump — forward messages from the hub's channel to the
    // websocket. Exits when the channel closes (hub unregisters the device
    // or the cancel token fires).
    let write_cancel = cancel.clone();
    let mut write_sender = sender;
    let write_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = write_cancel.cancelled() => break,
                msg = rx.recv() => {
                    let Some(msg) = msg else { break; };
                    let text = match serde_json::to_string(&msg) {
                        Ok(t) => t,
                        Err(e) => {
                            tracing::error!(error = %e, "failed to serialise server message");
                            continue;
                        }
                    };
                    if write_sender.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
            }
        }
        // Close our half of the socket so the read side sees EOF.
        let _ = write_sender.close().await;
    });

    // Step 4: read pump — dispatch client messages to the hub.
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let parsed: Result<ClientMessage, _> = serde_json::from_str(&text);
                let client_msg = match parsed {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::debug!(error = %e, "invalid client message");
                        let _ = hub
                            .route_error(
                                user_id,
                                &device_id,
                                ErrorCode::InvalidMessage,
                                e.to_string(),
                            )
                            .await;
                        continue;
                    }
                };
                dispatch(user_id, &device_id, client_msg, &hub).await;
            }
            Ok(Message::Close(_)) | Err(_) => break,
            // Ignore binary/ping/pong frames; the websocket layer handles
            // protocol-level ping/pong already.
            Ok(_) => continue,
        }
    }

    // Step 5: tear down. Cancel the write task (it'll exit on next loop
    // tick) and unregister the device from the hub.
    cancel.cancel();
    let _ = write_task.await;
    hub.unregister(user_id, &device_id).await;
}

async fn read_identify(
    receiver: &mut futures::stream::SplitStream<WebSocket>,
) -> Result<Identify, &'static str> {
    let first = timeout(IDENTIFY_TIMEOUT, receiver.next())
        .await
        .map_err(|_| "identify timeout")?
        .ok_or("client closed before identify")?
        .map_err(|_| "ws error before identify")?;

    let text = match first {
        Message::Text(t) => t,
        Message::Close(_) => return Err("client closed before identify"),
        _ => return Err("first message must be identify (text)"),
    };

    let parsed: Result<ClientMessage, _> = serde_json::from_str(&text);
    match parsed {
        Ok(ClientMessage::Identify(identify)) => Ok(identify),
        Ok(_) => Err("first message must be identify"),
        Err(e) => {
            tracing::debug!(error = %e, "identify parse error");
            Err("invalid identify")
        }
    }
}

async fn send_error_and_close(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    code: ErrorCode,
    message: &str,
) -> Result<(), axum::Error> {
    let payload = json!({
        "type": "error",
        "code": code.to_string(),
        "message": message,
    })
    .to_string();
    let _ = sender.send(Message::Text(payload)).await;
    let _ = sender
        .send(Message::Close(Some(CloseFrame {
            code: axum::extract::ws::close_code::INVALID,
            reason: "identify failed".into(),
        })))
        .await;
    Ok(())
}

async fn dispatch(
    user_id: i32,
    from: &super::protocol::DeviceId,
    msg: ClientMessage,
    hub: &Arc<Hub>,
) {
    match msg {
        ClientMessage::Identify(_) => {
            // Identify is only valid as the first message. Subsequent
            // identifies are ignored.
        }
        ClientMessage::PublishState { state } => {
            hub.publish_state(user_id, from, state).await;
        }
        ClientMessage::Command { command } => {
            let res = hub.route_command(user_id, from, command).await;
            if matches!(res, super::hub::CommandResult::NoActivePlayer) {
                let _ = hub
                    .route_error(user_id, from, ErrorCode::NoActivePlayer, "no active player")
                    .await;
            }
        }
        ClientMessage::Transfer { to_device_id } => {
            let res = hub.transfer(user_id, &to_device_id).await;
            if matches!(res, super::hub::TransferResult::UnknownDevice) {
                let _ = hub
                    .route_error(
                        user_id,
                        from,
                        ErrorCode::UnknownDevice,
                        "target device not found",
                    )
                    .await;
            }
        }
        ClientMessage::Heartbeat => {
            hub.touch(user_id, from).await;
        }
    }
}
