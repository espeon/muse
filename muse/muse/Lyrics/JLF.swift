import Foundation

// MARK: - JLF (JSON Lyrics Format)

struct JLF: Codable, Sendable {
    var lines: SyncedLines
    var richsync: SyncedRich?
    var metadata: SyncedMetadata?
    var source: String
    var name: String?
    var message: String?
}

struct SyncedMetadata: Codable, Sendable {
    var mxmId: String?
    var iTunesId: String?
    var spotifyId: String?
    var artist: String
    var title: String
    var album: String
    var copyright: String?

    enum CodingKeys: String, CodingKey {
        case mxmId = "MxmId"
        case iTunesId = "ITunesId"
        case spotifyId = "SpotifyId"
        case artist = "Artist"
        case title = "Title"
        case album = "Album"
        case copyright = "Copyright"
    }
}

struct SyncedLines: Codable, Sendable {
    var lines: [SyncedLine]
    var linesEnd: Int
}

struct SyncedLine: Codable, Sendable, Identifiable {
    var time: Int  // ms since start of song
    var text: String
    var translation: String?

    var id: Int { time }
}

// MARK: - Rich Sync (for future use)

struct SyncedRich: Codable, Sendable {
    var totalTime: Int
    var sections: [SyncedRichSection]
    var agents: [SyncedRichAgent]
}

enum SyncedRichAgentType: String, Codable, Sendable {
    case person
    case group
    case other
}

struct SyncedRichAgent: Codable, Sendable {
    var type: String
    var id: String
}

struct SyncedRichSection: Codable, Sendable {
    var timeStart: Int
    var timeEnd: Int
    var lines: [SyncedRichLine]
}

struct SyncedRichLine: Codable, Sendable {
    var timeStart: Int
    var timeEnd: Int
    var text: String
    var segments: [SyncedRichLineSegment]
    var agent: String
    var bgVox: SyncedRichBackgroundLine?
}

struct SyncedRichBackgroundLine: Codable, Sendable {
    var timeStart: Int
    var timeEnd: Int
    var text: String
    var segments: [SyncedRichLineSegment]
}

struct SyncedRichLineSegment: Codable, Sendable {
    var text: String
    var timeStart: Int
    var timeEnd: Int
}
