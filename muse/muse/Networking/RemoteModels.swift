//
//  RemoteModels.swift
//  muse
//
//  Wire types matching maki's remote/WS protocol (maki/src/api/remote/protocol.rs).
//  Naming uses camelCase; a JSONEncoder/Decoder with the snake-case key
//  strategy handles the wire format automatically.
//
//  For tagged unions (ClientMessage, ServerMessage, Command) we use a
//  struct with a `type`/`kind` discriminator and optional fields. Outbound
//  encoding omits nil fields so we don't send fields that the Rust side
//  would reject as unknown. Inbound decoding uses synthesized Codable with
//  `decodeIfPresent` which is tolerant of both missing and explicit-null
//  fields.
//

import Foundation

// MARK: - Queue

struct RemoteQueueItem: Codable, Equatable, Identifiable {
    var itemId: String
    var trackId: Int

    var id: String { itemId }
}

struct RemotePlaybackState: Codable, Equatable {
    var currentItemId: String?
    var positionMs: Int64
    var isPlaying: Bool
    var queue: [RemoteQueueItem]
    var updatedAt: Int64

    static func idle() -> RemotePlaybackState {
        RemotePlaybackState(
            currentItemId: nil,
            positionMs: 0,
            isPlaying: false,
            queue: [],
            updatedAt: Int64(Date().timeIntervalSince1970 * 1000)
        )
    }
}

// MARK: - Devices

enum RemoteDeviceKind: String, Codable {
    case ios
    case android
    case web
    case server
}

struct RemoteDevice: Codable, Equatable, Identifiable {
    var deviceId: String
    var name: String
    var kind: RemoteDeviceKind
    var isActivePlayer: Bool
    var lastSeen: Int64

    var id: String { deviceId }
}

// MARK: - Commands (outbound)

struct RemoteCommand: Codable, Equatable {
    enum Kind: String, Codable {
        case play
        case pause
        case toggle
        case next
        case previous
        case seek
        case setQueue = "set_queue"
        case addToQueue = "add_to_queue"
        case removeFromQueue = "remove_from_queue"
        case reorderQueue = "reorder_queue"
    }

    var kind: Kind
    var positionMs: Int64?
    var trackIds: [Int]?
    var startIndex: Int?
    var trackId: Int?
    var afterItemId: String?
    var itemId: String?

    enum CodingKeys: String, CodingKey {
        case kind
        case positionMs = "position_ms"
        case trackIds = "track_ids"
        case startIndex = "start_index"
        case trackId = "track_id"
        case afterItemId = "after_item_id"
        case itemId = "item_id"
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        try c.encode(self.kind, forKey: .kind)
        switch self.kind {
        case .play, .pause, .toggle, .next, .previous:
            return
        case .seek:
            try c.encodeIfPresent(self.positionMs, forKey: .positionMs)
        case .setQueue:
            try c.encodeIfPresent(self.trackIds, forKey: .trackIds)
            try c.encodeIfPresent(self.startIndex, forKey: .startIndex)
        case .addToQueue:
            try c.encodeIfPresent(self.trackId, forKey: .trackId)
            try c.encodeIfPresent(self.afterItemId, forKey: .afterItemId)
        case .removeFromQueue:
            try c.encodeIfPresent(self.itemId, forKey: .itemId)
        case .reorderQueue:
            try c.encodeIfPresent(self.itemId, forKey: .itemId)
            try c.encodeIfPresent(self.afterItemId, forKey: .afterItemId)
        }
    }
}

extension RemoteCommand {
    static let play = RemoteCommand(kind: .play)
    static let pause = RemoteCommand(kind: .pause)
    static let toggle = RemoteCommand(kind: .toggle)
    static let next = RemoteCommand(kind: .next)
    static let previous = RemoteCommand(kind: .previous)

    static func seek(positionMs: Int64) -> RemoteCommand {
        RemoteCommand(kind: .seek, positionMs: positionMs)
    }
    static func setQueue(trackIds: [Int], startIndex: Int) -> RemoteCommand {
        RemoteCommand(kind: .setQueue, trackIds: trackIds, startIndex: startIndex)
    }
    static func addToQueue(trackId: Int, afterItemId: String? = nil) -> RemoteCommand {
        RemoteCommand(kind: .addToQueue, trackId: trackId, afterItemId: afterItemId)
    }
    static func removeFromQueue(itemId: String) -> RemoteCommand {
        RemoteCommand(kind: .removeFromQueue, itemId: itemId)
    }
    static func reorderQueue(itemId: String, afterItemId: String? = nil) -> RemoteCommand {
        // Memberwise init follows declaration order: trackId, afterItemId, itemId.
        // For reorder we leave trackId nil; itemId is the one being moved.
        RemoteCommand(kind: .reorderQueue, trackId: nil, afterItemId: afterItemId, itemId: itemId)
    }
}

// MARK: - Client messages (outbound)

struct RemoteClientMessage: Codable {
    enum MessageType: String, Codable {
        case identify
        case publishState = "publish_state"
        case command
        case transfer
        case heartbeat
    }

    var type: MessageType
    var deviceId: String?
    var name: String?
    var kind: RemoteDeviceKind?
    var state: RemotePlaybackState?
    var command: RemoteCommand?
    var toDeviceId: String?

    enum CodingKeys: String, CodingKey {
        case type
        case deviceId = "device_id"
        case name
        case kind
        case state
        case command
        case toDeviceId = "to_device_id"
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        try c.encode(self.type, forKey: .type)
        try c.encodeIfPresent(self.deviceId, forKey: .deviceId)
        try c.encodeIfPresent(self.name, forKey: .name)
        try c.encodeIfPresent(self.kind, forKey: .kind)
        try c.encodeIfPresent(self.state, forKey: .state)
        try c.encodeIfPresent(self.command, forKey: .command)
        try c.encodeIfPresent(self.toDeviceId, forKey: .toDeviceId)
    }

    static func identify(deviceId: String, name: String, kind: RemoteDeviceKind) -> RemoteClientMessage {
        RemoteClientMessage(type: .identify, deviceId: deviceId, name: name, kind: kind)
    }
    static func publishState(_ state: RemotePlaybackState) -> RemoteClientMessage {
        RemoteClientMessage(type: .publishState, state: state)
    }
    static func command(_ command: RemoteCommand) -> RemoteClientMessage {
        RemoteClientMessage(type: .command, command: command)
    }
    static func transfer(toDeviceId: String) -> RemoteClientMessage {
        RemoteClientMessage(type: .transfer, toDeviceId: toDeviceId)
    }
    static let heartbeat = RemoteClientMessage(type: .heartbeat)
}

// MARK: - Server messages (inbound)

struct RemoteServerMessage: Codable {
    enum MessageType: String, Codable {
        case welcome
        case state
        case deviceList = "device_list"
        case command
        case requestPublish = "request_publish"
        case error
    }

    var type: MessageType
    var yourDeviceId: String?
    var activeDeviceId: String?
    var devices: [RemoteDevice]?
    var lastState: RemotePlaybackState?
    var state: RemotePlaybackState?
    var fromDeviceId: String?
    var seq: UInt64?
    var command: RemoteCommand?
    var code: String?
    var message: String?
}
