use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::protocol::{Command, DeviceId, DeviceKind, ErrorCode, PlaybackState, ServerMessage};
use super::state::UserSession;

/// Owns per-user playback sessions in memory. Shared via `Arc<Hub>` /
/// `axum::Extension`. Long-lived; lives for the process lifetime.
pub struct Hub {
    sessions: DashMap<i32, Mutex<UserSession>>,
    timeout: Duration,
    sweep_interval: Duration,
}

impl Hub {
    pub fn new() -> Self {
        Self::with_timeouts(Duration::from_secs(25), Duration::from_secs(5))
    }

    pub fn with_timeouts(device_timeout: Duration, sweep_interval: Duration) -> Self {
        Self {
            sessions: DashMap::new(),
            timeout: device_timeout,
            sweep_interval,
        }
    }

    /// Register a device on a user's session. Returns the receiver the WS
    /// write pump should drain, a cancellation token the WS read pump
    /// should select on, and the previous sender if this device_id was
    /// already connected (the caller should drop it to close the prior
    /// write pump).
    pub async fn register(
        &self,
        user_id: i32,
        device_id: DeviceId,
        name: String,
        kind: DeviceKind,
    ) -> RegisterHandle {
        let (tx, rx) = mpsc::channel(super::state::DEVICE_CHANNEL_CAPACITY);
        let now = now_millis();
        let session_lock = self.session_for(user_id);
        let mut session = session_lock.lock().await;
        let outcome = session.register(device_id.clone(), name, kind, tx, now);
        let cancel = session.cancel_token_for(&device_id).expect("just inserted");
        RegisterHandle {
            rx,
            cancel,
            old_sender: outcome.old_sender,
        }
    }

    /// Remove a device. If the device was the active player, the
    /// remaining devices are notified via DeviceList.
    pub async fn unregister(&self, user_id: i32, device_id: &DeviceId) {
        let outcome = {
            let Some(session_lock) = self.sessions.get(&user_id) else {
                return;
            };
            let mut session = session_lock.lock().await;
            let Some(outcome) = session.unregister(device_id) else {
                return;
            };
            if session.device_count() == 0 {
                drop(session);
                self.sessions.remove(&user_id);
                return;
            }
            outcome
        };
        let _ = outcome;
        self.broadcast_device_list(user_id).await;
    }

    pub async fn touch(&self, user_id: i32, device_id: &DeviceId) {
        let Some(session_lock) = self.sessions.get(&user_id) else {
            return;
        };
        let mut session = session_lock.lock().await;
        session.touch(device_id, now_millis());
    }

    /// Deliver an error message to a specific device. Used by the WS layer
    /// to report things like NoActivePlayer back to the requester.
    pub async fn route_error(
        &self,
        user_id: i32,
        device_id: &DeviceId,
        code: ErrorCode,
        message: impl Into<String>,
    ) {
        let Some(session_lock) = self.sessions.get(&user_id) else {
            return;
        };
        let session = session_lock.lock().await;
        if let Some(sender) = session.sender_for(device_id) {
            let _ = sender.try_send(ServerMessage::error(code, message));
        }
    }

    /// Apply a state publish from a device. Only the active player's
    /// publish is accepted; others are silently ignored (the caller can
    /// optionally send an error). The accepted state is broadcast as a
    /// State message to all *other* devices.
    pub async fn publish_state(
        &self,
        user_id: i32,
        from: &DeviceId,
        state: PlaybackState,
    ) -> PublishResult {
        let outcome = {
            let Some(session_lock) = self.sessions.get(&user_id) else {
                return PublishResult::NoSession;
            };
            let mut session = session_lock.lock().await;
            session.publish_state(from, state)
        };
        if !outcome.is_active_player {
            return PublishResult::NotActivePlayer;
        }
        let Some(msg) = outcome.broadcast else {
            return PublishResult::NotActivePlayer;
        };
        self.fan_out_except(user_id, from, &msg).await;
        PublishResult::Accepted
    }

    /// Route a command from a controller to the active player.
    pub async fn route_command(
        &self,
        user_id: i32,
        from: &DeviceId,
        command: Command,
    ) -> CommandResult {
        let (active_id, routed) = {
            let Some(session_lock) = self.sessions.get(&user_id) else {
                return CommandResult::NoSession;
            };
            let session = session_lock.lock().await;
            let Some(active) = session.active_device_id().cloned() else {
                return CommandResult::NoActivePlayer;
            };
            let Some(routed) = session.route_command(from, command) else {
                return CommandResult::NoActivePlayer;
            };
            (active, routed)
        };
        let Some(session_lock) = self.sessions.get(&user_id) else {
            return CommandResult::NoSession;
        };
        let session = session_lock.lock().await;
        if let Some(entry) = session.sender_for(&active_id) {
            let _ = entry.try_send(routed);
        }
        CommandResult::Routed
    }

    /// Transfer the active-player role to a new device. The new device
    /// is told it's active (and given the cached state); the old device
    /// (if still connected and different) is told to stop; all devices
    /// get a DeviceList.
    pub async fn transfer(&self, user_id: i32, to_device_id: &DeviceId) -> TransferResult {
        let (new_player_sender, old_player_sender, to_new_player, to_old_player, device_list) = {
            let Some(session_lock) = self.sessions.get(&user_id) else {
                return TransferResult::NoSession;
            };
            let mut session = session_lock.lock().await;
            let Some(outcome) = session.transfer(to_device_id) else {
                return TransferResult::UnknownDevice;
            };
            let new_player_sender = session.sender_for(to_device_id).cloned();
            let old_player_sender = outcome
                .old_device_id
                .as_ref()
                .and_then(|id| session.sender_for(id).cloned());
            (
                new_player_sender,
                old_player_sender,
                outcome.to_new_player,
                outcome.to_old_player,
                outcome.device_list,
            )
        };

        if let Some(sender) = new_player_sender {
            let _ = sender.try_send(to_new_player);
        }
        if let (Some(sender), Some(msg)) = (old_player_sender, to_old_player) {
            let _ = sender.try_send(msg);
        }

        // Broadcast the new device list to everyone. The new player
        // already got the same info via the welcome; receiving it again
        // is harmless.
        self.fan_out(user_id, &device_list).await;

        TransferResult::Routed
    }

    /// Iterate every device on every user session and disconnect any
    /// whose last_seen is older than the configured timeout.
    pub async fn sweep_stale(&self) {
        let now = now_millis();
        let timeout_ms = self.timeout.as_millis() as i64;
        let user_ids: Vec<i32> = self.sessions.iter().map(|e| *e.key()).collect();
        for user_id in user_ids {
            let stale = {
                let Some(session_lock) = self.sessions.get(&user_id) else {
                    continue;
                };
                let session = session_lock.lock().await;
                session.stale_devices(now, timeout_ms)
            };
            for device_id in stale {
                self.unregister(user_id, &device_id).await;
            }
        }
    }

    /// Spawn a background task that periodically sweeps stale devices.
    pub fn start_sweeper(self: &std::sync::Arc<Self>) -> JoinHandle<()> {
        let hub = std::sync::Arc::clone(self);
        let interval = self.sweep_interval;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                ticker.tick().await;
                hub.sweep_stale().await;
            }
        })
    }

    fn session_for(&self, user_id: i32) -> dashmap::mapref::one::Ref<'_, i32, Mutex<UserSession>> {
        // Ensure a session exists, then return a Ref to it. The RefMut
        // returned by `entry().or_insert_with` is dropped immediately; a
        // concurrent insert will block until that brief critical section
        // is over.
        if !self.sessions.contains_key(&user_id) {
            self.sessions
                .insert(user_id, Mutex::new(UserSession::new()));
        }
        self.sessions.get(&user_id).expect("just inserted")
    }

    async fn broadcast_device_list(&self, user_id: i32) {
        let msg = {
            let Some(session_lock) = self.sessions.get(&user_id) else {
                return;
            };
            let session = session_lock.lock().await;
            ServerMessage::DeviceList {
                devices: session.device_infos(),
                active_device_id: session.active_device_id().cloned(),
            }
        };
        self.fan_out(user_id, &msg).await;
    }

    async fn fan_out(&self, user_id: i32, msg: &ServerMessage) {
        let senders = {
            let Some(session_lock) = self.sessions.get(&user_id) else {
                return;
            };
            let session = session_lock.lock().await;
            session.sender_snapshots()
        };
        for (_, sender) in senders {
            // If a device's channel is closed, the write task has
            // already exited; the close path on the read side will
            // unregister it. We don't need to do anything here.
            let _ = sender.try_send(msg.clone());
        }
    }

    async fn fan_out_except(&self, user_id: i32, except: &DeviceId, msg: &ServerMessage) {
        let senders = {
            let Some(session_lock) = self.sessions.get(&user_id) else {
                return;
            };
            let session = session_lock.lock().await;
            session.sender_snapshots()
        };
        for (id, sender) in senders {
            if &id == except {
                continue;
            }
            let _ = sender.try_send(msg.clone());
        }
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RegisterHandle {
    pub rx: mpsc::Receiver<ServerMessage>,
    pub cancel: CancellationToken,
    pub old_sender: Option<mpsc::Sender<ServerMessage>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PublishResult {
    Accepted,
    NotActivePlayer,
    NoSession,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandResult {
    Routed,
    NoActivePlayer,
    NoSession,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransferResult {
    Routed,
    UnknownDevice,
    NoSession,
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::remote::protocol::{Command, PlaybackState, QueueItem};

    fn state(track: i32) -> PlaybackState {
        PlaybackState {
            current_item_id: Some(format!("i-{}", track)),
            position_ms: 100,
            is_playing: true,
            queue: vec![QueueItem {
                item_id: format!("i-{}", track),
                track_id: track,
            }],
            updated_at: 1,
        }
    }

    async fn drain_welcome(rx: &mut mpsc::Receiver<ServerMessage>) {
        let msg = rx.recv().await.expect("welcome");
        assert!(matches!(msg, ServerMessage::Welcome { .. }));
    }

    /// Drain every message currently buffered in `rx` (non-blocking).
    /// Tests use this between operations to clear the stream before
    /// asserting on the next operation's output.
    fn drain_now(rx: &mut mpsc::Receiver<ServerMessage>) -> Vec<ServerMessage> {
        let mut out = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            out.push(msg);
        }
        out
    }

    #[tokio::test]
    async fn register_sends_welcome() {
        let hub = std::sync::Arc::new(Hub::new());
        let h = hub
            .register(1, DeviceId::new("a"), "Phone".into(), DeviceKind::Ios)
            .await;
        let mut rx = h.rx;
        drain_welcome(&mut rx).await;
    }

    #[tokio::test]
    async fn publish_state_routes_to_other_devices() {
        let hub = std::sync::Arc::new(Hub::new());
        let h_a = hub
            .register(1, DeviceId::new("a"), "A".into(), DeviceKind::Ios)
            .await;
        let mut rx_a = h_a.rx;
        drain_welcome(&mut rx_a).await;
        let h_b = hub
            .register(1, DeviceId::new("b"), "B".into(), DeviceKind::Ios)
            .await;
        let mut rx_b = h_b.rx;
        drain_welcome(&mut rx_b).await;

        assert_eq!(
            hub.transfer(1, &DeviceId::new("a")).await,
            TransferResult::Routed
        );
        drain_now(&mut rx_a);
        drain_now(&mut rx_b);

        let r = hub.publish_state(1, &DeviceId::new("a"), state(5)).await;
        assert_eq!(r, PublishResult::Accepted);

        let msg = rx_b.recv().await.expect("b gets state");
        match msg {
            ServerMessage::State {
                state,
                from_device_id,
                seq,
            } => {
                assert_eq!(state.current_item_id, Some("i-5".into()));
                assert_eq!(state.queue[0].track_id, 5);
                assert_eq!(from_device_id, DeviceId::new("a"));
                assert!(seq > 0);
            }
            other => panic!("expected State, got {:?}", other),
        }
        // a should not receive its own state
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(
            rx_a.try_recv().is_err(),
            "a should not receive its own state"
        );
    }

    #[tokio::test]
    async fn publish_from_non_active_player_is_rejected() {
        let hub = std::sync::Arc::new(Hub::new());
        let _ = hub
            .register(1, DeviceId::new("a"), "A".into(), DeviceKind::Ios)
            .await;
        let _ = hub
            .register(1, DeviceId::new("b"), "B".into(), DeviceKind::Ios)
            .await;
        let _ = hub.transfer(1, &DeviceId::new("a")).await;
        let r = hub.publish_state(1, &DeviceId::new("b"), state(9)).await;
        assert_eq!(r, PublishResult::NotActivePlayer);
    }

    #[tokio::test]
    async fn route_command_with_no_active_player_errors() {
        let hub = std::sync::Arc::new(Hub::new());
        let _ = hub
            .register(1, DeviceId::new("a"), "A".into(), DeviceKind::Ios)
            .await;
        let r = hub
            .route_command(1, &DeviceId::new("a"), Command::Play)
            .await;
        assert_eq!(r, CommandResult::NoActivePlayer);
    }

    #[tokio::test]
    async fn route_command_to_active_player() {
        let hub = std::sync::Arc::new(Hub::new());
        let h_a = hub
            .register(1, DeviceId::new("a"), "A".into(), DeviceKind::Ios)
            .await;
        let mut rx_a = h_a.rx;
        drain_welcome(&mut rx_a).await;
        let _h_b = hub
            .register(1, DeviceId::new("b"), "B".into(), DeviceKind::Ios)
            .await;
        let _ = hub.transfer(1, &DeviceId::new("a")).await;
        drain_now(&mut rx_a);

        let r = hub
            .route_command(1, &DeviceId::new("b"), Command::Toggle)
            .await;
        assert_eq!(r, CommandResult::Routed);
        let msg = rx_a.recv().await.expect("a gets command");
        match msg {
            ServerMessage::Command {
                command,
                from_device_id,
            } => {
                assert_eq!(command, Command::Toggle);
                assert_eq!(from_device_id, DeviceId::new("b"));
            }
            other => panic!("expected Command, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn unregister_active_player_clears_active_and_broadcasts() {
        let hub = std::sync::Arc::new(Hub::new());
        let _ = hub
            .register(1, DeviceId::new("a"), "A".into(), DeviceKind::Ios)
            .await;
        let h_b = hub
            .register(1, DeviceId::new("b"), "B".into(), DeviceKind::Ios)
            .await;
        let mut rx_b = h_b.rx;
        drain_welcome(&mut rx_b).await;
        let _ = hub.transfer(1, &DeviceId::new("a")).await;
        drain_now(&mut rx_b);

        hub.unregister(1, &DeviceId::new("a")).await;
        let msg = rx_b.recv().await.expect("b gets device list");
        match msg {
            ServerMessage::DeviceList {
                devices,
                active_device_id,
            } => {
                assert_eq!(devices.len(), 1);
                assert!(active_device_id.is_none());
            }
            other => panic!("expected DeviceList, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn transfer_promotes_new_active() {
        let hub = std::sync::Arc::new(Hub::new());
        let h_a = hub
            .register(1, DeviceId::new("a"), "A".into(), DeviceKind::Ios)
            .await;
        let mut rx_a = h_a.rx;
        drain_welcome(&mut rx_a).await;
        let h_b = hub
            .register(1, DeviceId::new("b"), "B".into(), DeviceKind::Ios)
            .await;
        let mut rx_b = h_b.rx;
        drain_welcome(&mut rx_b).await;
        // Make a active so a transfer has a meaningful "old" player.
        let _ = hub.transfer(1, &DeviceId::new("a")).await;
        drain_now(&mut rx_a);
        drain_now(&mut rx_b);

        // Now transfer to b.
        let r = hub.transfer(1, &DeviceId::new("b")).await;
        assert_eq!(r, TransferResult::Routed);

        // a should get a device_list telling it it's no longer active.
        // a may receive multiple device_lists (one from to_old_player,
        // one from fan_out); assert the first one reflects the new state.
        let msg_a = rx_a.recv().await.expect("a gets device_list");
        match msg_a {
            ServerMessage::DeviceList {
                active_device_id, ..
            } => {
                assert_eq!(active_device_id, Some(DeviceId::new("b")));
            }
            other => panic!("expected DeviceList, got {:?}", other),
        }

        // b should get a welcome telling it it's now active
        let msg_b = rx_b.recv().await.expect("b gets welcome");
        match msg_b {
            ServerMessage::Welcome {
                active_device_id, ..
            } => {
                assert_eq!(active_device_id, Some(DeviceId::new("b")));
            }
            other => panic!("expected Welcome, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn transfer_to_unknown_device_errors() {
        let hub = std::sync::Arc::new(Hub::new());
        let _ = hub
            .register(1, DeviceId::new("a"), "A".into(), DeviceKind::Ios)
            .await;
        let r = hub.transfer(1, &DeviceId::new("ghost")).await;
        assert_eq!(r, TransferResult::UnknownDevice);
    }
}
