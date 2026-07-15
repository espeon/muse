use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::protocol::{Command, DeviceId, DeviceInfo, DeviceKind, PlaybackState, ServerMessage};

pub(crate) const DEVICE_CHANNEL_CAPACITY: usize = 32;

#[derive(Debug)]
pub struct DeviceEntry {
    pub name: String,
    pub kind: DeviceKind,
    pub sender: mpsc::Sender<ServerMessage>,
    pub last_seen: i64,
    /// Cancelled when this device's connection should terminate (e.g. the
    /// same device_id reconnected with a new connection; the old task must
    /// exit so it doesn't leak).
    pub cancel: CancellationToken,
}

impl DeviceEntry {
    pub fn device_info(&self, id: &DeviceId, is_active_player: bool) -> DeviceInfo {
        DeviceInfo {
            device_id: id.clone(),
            name: self.name.clone(),
            kind: self.kind,
            is_active_player,
            last_seen: self.last_seen,
        }
    }
}

/// Per-user playback session. In-memory only; not persisted.
#[derive(Debug, Default)]
pub struct UserSession {
    devices: HashMap<DeviceId, DeviceEntry>,
    active_device_id: Option<DeviceId>,
    last_state: Option<PlaybackState>,
    last_seq: u64,
}

#[derive(Debug)]
pub struct RegisterOutcome {
    /// Previous sender for this device_id, if any (reconnect). The caller
    /// should drop it to close the old write pump.
    pub old_sender: Option<mpsc::Sender<ServerMessage>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct UnregisterOutcome {
    /// True if the removed device was the active player.
    pub was_active_player: bool,
}

#[derive(Debug)]
pub struct PublishOutcome {
    /// The State message to broadcast to all *other* devices.
    pub broadcast: Option<ServerMessage>,
    /// True if the device is allowed to publish (is the active player).
    pub is_active_player: bool,
}

#[derive(Debug)]
pub struct TransferOutcome {
    /// The device that *was* the active player before this transfer, if any
    /// and if it is still connected and different from the new player.
    pub old_device_id: Option<DeviceId>,
    /// Message to send to the new active player (so it can start rendering).
    pub to_new_player: ServerMessage,
    /// Message to send to the previous active player (if different and still
    /// connected), telling it to stop.
    pub to_old_player: Option<ServerMessage>,
    /// DeviceList to broadcast to all remaining devices.
    pub device_list: ServerMessage,
}

impl UserSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_device_id(&self) -> Option<&DeviceId> {
        self.active_device_id.as_ref()
    }

    #[allow(dead_code)]
    pub fn last_state(&self) -> Option<&PlaybackState> {
        self.last_state.as_ref()
    }

    pub fn device_infos(&self) -> Vec<DeviceInfo> {
        self.devices
            .iter()
            .map(|(id, entry)| entry.device_info(id, Some(id) == self.active_device_id.as_ref()))
            .collect()
    }

    /// Register a device. If the same device_id is already present, the old
    /// entry's cancel token is triggered (caller's old task should exit) and
    /// the old sender is returned in `old_sender` so the caller can drop
    /// the prior write pump. The Welcome message is delivered directly to
    /// the new device's sender.
    pub fn register(
        &mut self,
        device_id: DeviceId,
        name: String,
        kind: DeviceKind,
        sender: mpsc::Sender<ServerMessage>,
        now: i64,
    ) -> RegisterOutcome {
        let new_entry = DeviceEntry {
            name,
            kind,
            sender: sender.clone(),
            last_seen: now,
            cancel: CancellationToken::new(),
        };

        let old_sender = self
            .devices
            .insert(device_id.clone(), new_entry)
            .map(|old| {
                old.cancel.cancel();
                old.sender
            });

        // Build welcome AFTER the insert so device_infos() includes this
        // device in the room view.
        let welcome = ServerMessage::Welcome {
            your_device_id: device_id,
            active_device_id: self.active_device_id.clone(),
            devices: self.device_infos(),
            last_state: self.last_state.clone(),
        };

        // Best-effort: if the channel is already full or closed (shouldn't
        // be, since we just created it), we drop the welcome. The caller
        // will see an empty stream and disconnect.
        let _ = sender.try_send(welcome);

        RegisterOutcome { old_sender }
    }

    /// Remove a device. Returns whether the device was the active player
    /// (so the caller knows whether to broadcast a "no active player"
    /// DeviceList). The removed-device's stored state is discarded.
    pub fn unregister(&mut self, device_id: &DeviceId) -> Option<UnregisterOutcome> {
        let removed = self.devices.remove(device_id)?;
        let was_active = self
            .active_device_id
            .as_ref()
            .map(|a| a == device_id)
            .unwrap_or(false);
        if was_active {
            self.active_device_id = None;
        }
        // Drop the removed entry; its sender going out of scope closes
        // the write pump on the remote side.
        let _ = removed;
        Some(UnregisterOutcome {
            was_active_player: was_active,
        })
    }

    pub fn touch(&mut self, device_id: &DeviceId, now: i64) {
        if let Some(entry) = self.devices.get_mut(device_id) {
            entry.last_seen = now;
        }
    }

    /// Update last_seen on every device. Returns ids whose last_seen is
    /// older than `now - timeout_ms`. Callers should remove them and
    /// broadcast a DeviceList.
    pub fn stale_devices(&self, now: i64, timeout_ms: i64) -> Vec<DeviceId> {
        self.devices
            .iter()
            .filter_map(|(id, e)| {
                if now - e.last_seen > timeout_ms {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Apply a state publish from a device. Only the active player's
    /// publish is accepted; others get an "you're not the player" error
    /// wrapped in the broadcast field's absence (caller decides what to do).
    pub fn publish_state(&mut self, from: &DeviceId, state: PlaybackState) -> PublishOutcome {
        let is_active = self
            .active_device_id
            .as_ref()
            .map(|a| a == from)
            .unwrap_or(false);
        if !is_active {
            return PublishOutcome {
                broadcast: None,
                is_active_player: false,
            };
        }
        self.last_state = Some(state.clone());
        self.last_seq = self.last_seq.wrapping_add(1);
        PublishOutcome {
            broadcast: Some(ServerMessage::State {
                state,
                from_device_id: from.clone(),
                seq: self.last_seq,
            }),
            is_active_player: true,
        }
    }

    /// Route a command from a controller to the active player. Returns the
    /// ServerMessage::Command to deliver, or None if there's no active
    /// player (caller can reply with NoActivePlayer error).
    pub fn route_command(&self, from: &DeviceId, command: Command) -> Option<ServerMessage> {
        let active = self.active_device_id.as_ref()?;
        if active == from {
            return None;
        }
        Some(ServerMessage::Command {
            command,
            from_device_id: from.clone(),
        })
    }

    /// Transfer the active-player role to a new device. The new device must
    /// already be registered. The active player (old) is told to stop; the
    /// new device is told it's active; everyone gets a DeviceList.
    pub fn transfer(&mut self, to_device_id: &DeviceId) -> Option<TransferOutcome> {
        if !self.devices.contains_key(to_device_id) {
            return None;
        }
        let old_active = self.active_device_id.clone();
        self.active_device_id = Some(to_device_id.clone());

        let device_list = ServerMessage::DeviceList {
            devices: self.device_infos(),
            active_device_id: self.active_device_id.clone(),
        };

        // Tell the new player it's now active, and hand it the cached state
        // so it can resume. We use a fresh Welcome that mirrors the
        // post-transfer room view; the client's `your_device_id` will match
        // its own id, and `active_device_id` will equal that too, signalling
        // it should take over rendering.
        let to_new_player = ServerMessage::Welcome {
            your_device_id: to_device_id.clone(),
            active_device_id: self.active_device_id.clone(),
            devices: self.device_infos(),
            last_state: self.last_state.clone(),
        };

        let (old_device_id, to_old_player) = if let Some(old) = old_active.as_ref() {
            if old != to_device_id && self.devices.contains_key(old) {
                (Some(old.clone()), Some(device_list.clone()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        Some(TransferOutcome {
            old_device_id,
            to_new_player,
            to_old_player,
            device_list,
        })
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn sender_for(&self, device_id: &DeviceId) -> Option<&mpsc::Sender<ServerMessage>> {
        self.devices.get(device_id).map(|e| &e.sender)
    }

    pub fn cancel_token_for(&self, device_id: &DeviceId) -> Option<CancellationToken> {
        self.devices.get(device_id).map(|e| e.cancel.clone())
    }

    pub fn sender_snapshots(&self) -> Vec<(DeviceId, mpsc::Sender<ServerMessage>)> {
        self.devices
            .iter()
            .map(|(id, e)| (id.clone(), e.sender.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::remote::protocol::{PlaybackState, QueueItem};

    fn channel() -> (mpsc::Sender<ServerMessage>, mpsc::Receiver<ServerMessage>) {
        mpsc::channel(DEVICE_CHANNEL_CAPACITY)
    }

    fn now() -> i64 {
        1_000_000
    }

    fn dev_a() -> DeviceId {
        DeviceId::new("dev-a")
    }
    fn dev_b() -> DeviceId {
        DeviceId::new("dev-b")
    }

    fn state(track: i32) -> PlaybackState {
        PlaybackState {
            current_item_id: Some(format!("i-{}", track)),
            position_ms: 100,
            is_playing: true,
            queue: vec![QueueItem {
                item_id: format!("i-{}", track),
                track_id: track,
            }],
            updated_at: now(),
        }
    }

    #[test]
    fn register_emits_welcome_with_empty_room() {
        let mut s = UserSession::new();
        let (tx, mut rx) = channel();
        let out = s.register(dev_a(), "Phone".into(), DeviceKind::Ios, tx, now());

        assert!(out.old_sender.is_none());
        let msg = rx.try_recv().expect("welcome should be in channel");
        match msg {
            ServerMessage::Welcome {
                your_device_id,
                active_device_id,
                devices,
                last_state,
            } => {
                assert_eq!(your_device_id, dev_a());
                assert!(active_device_id.is_none());
                assert_eq!(devices.len(), 1);
                assert!(!devices[0].is_active_player);
                assert!(last_state.is_none());
            }
            other => panic!("expected Welcome, got {:?}", other),
        }
    }

    #[test]
    fn register_replaces_existing_device_sender() {
        let mut s = UserSession::new();
        let (tx1, _rx1) = channel();
        s.register(dev_a(), "Phone".into(), DeviceKind::Ios, tx1, now());
        let (tx2, _rx2) = channel();
        let out = s.register(dev_a(), "Phone".into(), DeviceKind::Ios, tx2, now());
        assert!(out.old_sender.is_some(), "old sender should be returned");
    }

    #[test]
    fn publish_state_from_non_active_player_is_rejected() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        let (tx_b, _rx_b) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.register(dev_b(), "B".into(), DeviceKind::Ios, tx_b, now());
        s.active_device_id = Some(dev_a());
        let out = s.publish_state(&dev_b(), state(1));
        assert!(!out.is_active_player);
        assert!(out.broadcast.is_none());
    }

    #[test]
    fn publish_state_from_active_player_increments_seq_and_broadcasts() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        let (tx_b, mut rx_b) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.register(dev_b(), "B".into(), DeviceKind::Ios, tx_b, now());
        s.active_device_id = Some(dev_a());

        // drain the welcome that landed in b's channel during register
        let welcome = rx_b.try_recv().expect("welcome for b");
        assert!(matches!(welcome, ServerMessage::Welcome { .. }));

        let out = s.publish_state(&dev_a(), state(7));
        assert!(out.is_active_player);
        let msg = out.broadcast.expect("broadcast expected");
        match &msg {
            ServerMessage::State {
                state,
                from_device_id,
                seq,
            } => {
                assert_eq!(state.current_item_id, Some("i-7".into()));
                assert_eq!(state.queue.len(), 1);
                assert_eq!(state.queue[0].track_id, 7);
                assert_eq!(from_device_id, &dev_a());
                assert_eq!(*seq, 1);
            }
            _ => panic!("expected State"),
        }
        // The fan-out is the hub's job; here we just assert the device_b
        // has no message yet (we didn't call fan_out).
        assert!(rx_b.try_recv().is_err());
    }

    #[test]
    fn publish_state_seq_is_monotonic_across_publishes() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.active_device_id = Some(dev_a());
        s.publish_state(&dev_a(), state(1));
        s.publish_state(&dev_a(), state(2));
        s.publish_state(&dev_a(), state(3));
        assert_eq!(s.last_seq, 3);
    }

    #[test]
    fn route_command_with_no_active_player_returns_none() {
        let s = UserSession::new();
        let routed = s.route_command(&dev_a(), Command::Play);
        assert!(routed.is_none());
    }

    #[test]
    fn route_command_from_active_player_returns_none() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.active_device_id = Some(dev_a());
        assert!(s.route_command(&dev_a(), Command::Play).is_none());
    }

    #[test]
    fn route_command_from_controller_goes_to_active_player() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        let (tx_b, _rx_b) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.register(dev_b(), "B".into(), DeviceKind::Ios, tx_b, now());
        s.active_device_id = Some(dev_a());

        let routed = s
            .route_command(&dev_b(), Command::Toggle)
            .expect("should route");
        match routed {
            ServerMessage::Command {
                command,
                from_device_id,
            } => {
                assert_eq!(command, Command::Toggle);
                assert_eq!(from_device_id, dev_b());
            }
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn transfer_to_unknown_device_returns_none() {
        let mut s = UserSession::new();
        assert!(s.transfer(&DeviceId::new("ghost")).is_none());
    }

    #[test]
    fn transfer_promotes_new_active_and_clears_old() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        let (tx_b, _rx_b) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.register(dev_b(), "B".into(), DeviceKind::Ios, tx_b, now());
        s.active_device_id = Some(dev_a());
        s.publish_state(&dev_a(), state(1)); // populate last_state

        let out = s.transfer(&dev_b()).expect("transfer ok");
        assert_eq!(s.active_device_id(), Some(&dev_b()));

        match out.to_new_player {
            ServerMessage::Welcome {
                your_device_id,
                active_device_id,
                last_state,
                ..
            } => {
                assert_eq!(your_device_id, dev_b());
                assert_eq!(active_device_id, Some(dev_b()));
                assert!(last_state.is_some());
            }
            _ => panic!("expected Welcome for new player"),
        }
        assert!(out.to_old_player.is_some(), "old player still connected");
    }

    #[test]
    fn transfer_to_same_device_is_a_noop_for_old_player() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.active_device_id = Some(dev_a());

        let out = s.transfer(&dev_a()).expect("transfer ok");
        assert!(out.to_old_player.is_none());
        assert_eq!(s.active_device_id(), Some(&dev_a()));
    }

    #[test]
    fn unregister_active_player_clears_active() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.active_device_id = Some(dev_a());

        let out = s.unregister(&dev_a()).expect("removed");
        assert!(out.was_active_player);
        assert!(s.active_device_id().is_none());
        assert!(s.device_count() == 0);
    }

    #[test]
    fn unregister_non_active_player_keeps_active() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        let (tx_b, _rx_b) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, now());
        s.register(dev_b(), "B".into(), DeviceKind::Ios, tx_b, now());
        s.active_device_id = Some(dev_a());

        let out = s.unregister(&dev_b()).expect("removed");
        assert!(!out.was_active_player);
        assert_eq!(s.active_device_id(), Some(&dev_a()));
    }

    #[test]
    fn stale_devices_detects_timed_out() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        let (tx_b, _rx_b) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, 100);
        s.register(dev_b(), "B".into(), DeviceKind::Ios, tx_b, 100);

        s.touch(&dev_a(), 1000);
        s.touch(&dev_b(), 200);

        // timeout 500ms
        let stale = s.stale_devices(1000, 500);
        assert_eq!(stale, vec![dev_b()]);
    }

    #[test]
    fn touch_updates_last_seen() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, 100);
        s.touch(&dev_a(), 800);
        let stale = s.stale_devices(1000, 300);
        assert!(
            stale.is_empty(),
            "device touched recently should not be stale"
        );
    }

    #[test]
    fn device_infos_reflect_active_player() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        let (tx_b, _rx_b) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, 0);
        s.register(dev_b(), "B".into(), DeviceKind::Ios, tx_b, 0);
        s.active_device_id = Some(dev_a());

        let infos = s.device_infos();
        assert_eq!(infos.len(), 2);
        let a = infos.iter().find(|i| i.device_id == dev_a()).unwrap();
        let b = infos.iter().find(|i| i.device_id == dev_b()).unwrap();
        assert!(a.is_active_player);
        assert!(!b.is_active_player);
    }

    #[test]
    fn identify_welcome_includes_cached_state() {
        let mut s = UserSession::new();
        let (tx_a, _rx_a) = channel();
        s.register(dev_a(), "A".into(), DeviceKind::Ios, tx_a, 0);
        s.active_device_id = Some(dev_a());
        s.publish_state(&dev_a(), state(9));
        assert!(s.last_state().is_some());

        let (tx_b, mut rx_b) = channel();
        let out = s.register(dev_b(), "B".into(), DeviceKind::Ios, tx_b, 0);
        assert!(out.old_sender.is_none());
        let msg = rx_b.try_recv().expect("welcome expected");
        match msg {
            ServerMessage::Welcome { last_state, .. } => {
                let s = last_state.unwrap();
                assert_eq!(s.current_item_id, Some("i-9".into()));
                assert_eq!(s.queue[0].track_id, 9);
            }
            _ => panic!("expected Welcome"),
        }
    }
}
