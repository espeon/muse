//
//  RemoteClient.swift
//  muse
//
//  WebSocket client for the "muse remote" feature (the eli/remote
//  protocol on maki). Holds the persistent connection to
//  `wss://<server>/api/v1/remote/ws`, sends IDENTIFY, receives Welcome
//  and live state updates, and lets the player publish its state or
//  send commands to the active player.
//
//  Mirrors the closure-based pattern of `APIClient` so token rotation
//  (auth refresh) is picked up automatically without re-construction.
//

import Foundation
import Observation
import UIKit

enum RemoteError: LocalizedError {
    case noServerURL
    case notAuthenticated
    case connectionFailed(String)
    case notConnected
    case sendFailed(String)

    var errorDescription: String? {
        switch self {
        case .noServerURL: return "Set a server URL first."
        case .notAuthenticated: return "Sign in to connect."
        case .connectionFailed(let m): return "Connection failed: \(m)"
        case .notConnected: return "Not connected to remote."
        case .sendFailed(let m): return "Send failed: \(m)"
        }
    }
}

@MainActor
@Observable
final class RemoteClient {

    enum ConnectionState: Equatable {
        case disconnected
        case connecting
        case connected
        case failed(String)
    }

    // MARK: - Observable state

    private(set) var connectionState: ConnectionState = .disconnected
    private(set) var myDeviceId: String = ""
    private(set) var myDeviceName: String = ""
    private(set) var activeDeviceId: String?
    private(set) var devices: [RemoteDevice] = []
    private(set) var lastState: RemotePlaybackState?
    private(set) var lastError: String?
    private(set) var lastSeq: UInt64 = 0

    /// True iff this device is the active player. The PlayerEngine
    /// observes this to decide whether to publish state and to know when
    /// to stop local playback.
    var isActivePlayer: Bool {
        guard !myDeviceId.isEmpty else { return false }
        return activeDeviceId == myDeviceId
    }

    /// Hook for the PlayerEngine to receive routed commands. Set once
    /// after construction. The handler runs on the main actor.
    var onIncomingCommand: ((RemoteCommand) -> Void)?

    /// Hook for the PlayerEngine to be notified when this device's
    /// active-player status changes (so it can stop/resume).
    var onActivePlayerChanged: ((Bool) -> Void)?

    // MARK: - Init

    private let serverURLProvider: () -> String?
    private let authHeaderProvider: () -> String?

    init(
        serverURL: @escaping () -> String?,
        authHeader: @escaping () -> String?
    ) {
        self.serverURLProvider = serverURL
        self.authHeaderProvider = authHeader
        self.myDeviceId = Self.loadOrCreateDeviceId()
        self.myDeviceName = UIDevice.current.name
    }

    /// Preview-only mutator that sets the observable state to a
    /// fixture. Not used in production. The leading underscore is the
    /// convention for "not a normal API"; kept un-guarded so SwiftUI
    /// previews work in any build configuration (Debug or Release).
    func _setPreviewState(
        connectionState: ConnectionState? = nil,
        myDeviceId: String? = nil,
        activeDeviceId: String?? = nil,
        devices: [RemoteDevice]? = nil,
        lastState: RemotePlaybackState?? = nil
    ) {
        if let s = connectionState { self.connectionState = s }
        if let id = myDeviceId { self.myDeviceId = id }
        if let id = activeDeviceId { self.activeDeviceId = id }
        if let d = devices { self.devices = d }
        if let s = lastState { self.lastState = s }
    }

    // MARK: - Lifecycle

    /// Open the connection. Idempotent. Begins the IDENTIFY handshake.
    /// Safe to call before authenticated; it'll be a no-op until both
    /// serverURL and an auth header are available.
    func start() {
        guard connectionState != .connecting, connectionState != .connected else { return }
        connectionState = .connecting
        connect()
    }

    /// Close the connection and don't reconnect.
    func shutdown() {
        explicitShutdown = true
        receiveTask?.cancel()
        receiveTask = nil
        heartbeatTask?.cancel()
        heartbeatTask = nil
        socketTask?.cancel(with: .goingAway, reason: nil)
        socketTask = nil
        connectionState = .disconnected
        activeDeviceId = nil
        devices = []
        lastState = nil
        lastSeq = 0
    }

    // MARK: - Send

    /// Publish this device's playback state. Only meaningful when this
    /// device is the active player (the server drops publishes from
    /// non-active devices).
    func publishState(_ state: RemotePlaybackState) async {
        await send(.publishState(state))
    }

    /// Send a command to the active player. The server routes it.
    /// No-op if there is no active player.
    func sendCommand(_ command: RemoteCommand) async {
        await send(.command(command))
    }

    /// Request that the given device become the active player. Pass
    /// `myDeviceId` to take over (the "Play on this device" action).
    func sendTransfer(to deviceId: String) async {
        await send(.transfer(toDeviceId: deviceId))
    }

    // MARK: - Internals

    private var socketTask: URLSessionWebSocketTask?
    private var receiveTask: Task<Void, Never>?
    private var heartbeatTask: Task<Void, Never>?
    private var reconnectAttempt: Int = 0
    private var explicitShutdown: Bool = false

    private let session: URLSession = {
        let cfg = URLSessionConfiguration.default
        cfg.waitsForConnectivity = true
        return URLSession(configuration: cfg)
    }()

    private let encoder: JSONEncoder = {
        let e = JSONEncoder()
        e.keyEncodingStrategy = .convertToSnakeCase
        return e
    }()

    private let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        return d
    }()

    private func connect() {
        guard let serverURLString = serverURLProvider(), !serverURLString.isEmpty else {
            connectionState = .failed("No server URL")
            return
        }
        guard let auth = authHeaderProvider() else {
            connectionState = .failed("Not authenticated")
            return
        }
        guard let url = makeWebSocketURL(serverURL: serverURLString) else {
            connectionState = .failed("Invalid server URL")
            return
        }

        var request = URLRequest(url: url)
        request.setValue(auth, forHTTPHeaderField: "Authorization")
        let task = session.webSocketTask(with: request)
        self.socketTask = task
        task.resume()

        // Send IDENTIFY as the first message.
        let identify = RemoteClientMessage.identify(
            deviceId: myDeviceId,
            name: myDeviceName,
            kind: .ios
        )
        Task { await self.write(identify) }

        // Begin the receive loop.
        receiveTask?.cancel()
        receiveTask = Task { [weak self] in
            await self?.runReceiveLoop()
        }

        // Begin heartbeats.
        heartbeatTask?.cancel()
        heartbeatTask = Task { [weak self] in
            await self?.runHeartbeatLoop()
        }
    }

    private func runReceiveLoop() async {
        guard let task = socketTask else { return }
        while !Task.isCancelled {
            do {
                let message = try await task.receive()
                await handleIncoming(message)
            } catch {
                await handleDisconnect(error: error)
                return
            }
        }
    }

    private func runHeartbeatLoop() async {
        while !Task.isCancelled {
            try? await Task.sleep(nanoseconds: 10 * 1_000_000_000) // 10s
            if Task.isCancelled { return }
            await send(.heartbeat)
        }
    }

    private func handleIncoming(_ message: URLSessionWebSocketTask.Message) async {
        let text: String
        switch message {
        case .string(let s): text = s
        case .data(let d):
            guard let s = String(data: d, encoding: .utf8) else { return }
            text = s
        @unknown default: return
        }
        guard let data = text.data(using: .utf8) else { return }
        let parsed: RemoteServerMessage
        do {
            parsed = try decoder.decode(RemoteServerMessage.self, from: data)
        } catch {
            lastError = "Bad message: \(error.localizedDescription)"
            return
        }
        applyServerMessage(parsed)
    }

    private func applyServerMessage(_ msg: RemoteServerMessage) {
        switch msg.type {
        case .welcome:
            if let id = msg.yourDeviceId { myDeviceId = id }
            let prevActive = activeDeviceId
            activeDeviceId = msg.activeDeviceId
            devices = msg.devices ?? []
            lastState = msg.lastState
            lastSeq = 0
            connectionState = .connected
            reconnectAttempt = 0
            lastError = nil
            if prevActive != activeDeviceId {
                onActivePlayerChanged?(isActivePlayer)
            }
        case .state:
            guard let state = msg.state else { return }
            // Drop out-of-order / late frames by monotonic seq.
            if let seq = msg.seq, seq <= lastSeq, lastSeq != 0 { return }
            lastState = state
            if let seq = msg.seq { lastSeq = seq }
        case .deviceList:
            let prevActive = activeDeviceId
            activeDeviceId = msg.activeDeviceId
            devices = msg.devices ?? []
            if prevActive != activeDeviceId {
                onActivePlayerChanged?(isActivePlayer)
            }
        case .command:
            if let command = msg.command {
                onIncomingCommand?(command)
            }
        case .requestPublish:
            // Server is asking the active player to re-publish. The
            // PlayerEngine handles this indirectly by publishing on its
            // own periodic timer; nothing to do here for v1.
            break
        case .error:
            lastError = msg.message ?? msg.code ?? "Unknown error"
        }
    }

    private func handleDisconnect(error: Error) async {
        socketTask = nil
        receiveTask = nil
        heartbeatTask = nil
        if explicitShutdown {
            return
        }
        connectionState = .failed(error.localizedDescription)
        scheduleReconnect()
    }

    private func scheduleReconnect() {
        guard !explicitShutdown else { return }
        let backoff = min(30, 1 << min(reconnectAttempt, 5))
        reconnectAttempt += 1
        Task { [weak self] in
            try? await Task.sleep(nanoseconds: UInt64(backoff) * 1_000_000_000)
            guard let self else { return }
            if self.explicitShutdown { return }
            self.connect()
        }
    }

    private func send(_ message: RemoteClientMessage) async {
        guard let task = socketTask else { return }
        do {
            let data = try encoder.encode(message)
            guard let text = String(data: data, encoding: .utf8) else { return }
            try await task.send(.string(text))
        } catch {
            lastError = error.localizedDescription
        }
    }

    private func write(_ message: RemoteClientMessage) async {
        // Used internally for the IDENTIFY; behaves like send but the
        // method name is private and doesn't need the same error
        // treatment since failure here is just a retry trigger.
        guard let task = socketTask else { return }
        do {
            let data = try encoder.encode(message)
            guard let text = String(data: data, encoding: .utf8) else { return }
            try await task.send(.string(text))
        } catch {
            // The receive loop will see the disconnect and reconnect.
        }
    }

    private func makeWebSocketURL(serverURL: String) -> URL? {
        var components = URLComponents(string: serverURL)
        switch components?.scheme {
        case "https": components?.scheme = "wss"
        case "http": components?.scheme = "ws"
        default: return nil
        }
        components?.path = "/api/v1/remote/ws"
        return components?.url
    }

    // MARK: - Device id persistence

    private static let deviceIdKey = "muse.remoteDeviceId"

    private static func loadOrCreateDeviceId() -> String {
        if let existing = KeychainHelper.load(forKey: deviceIdKey), !existing.isEmpty {
            return existing
        }
        let new = UUID().uuidString.lowercased()
        KeychainHelper.save(new, forKey: deviceIdKey)
        return new
    }
}
