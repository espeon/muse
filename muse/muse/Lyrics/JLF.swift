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

    init(lines: [SyncedLine], linesEnd: Int) {
        self.lines = lines
        self.linesEnd = linesEnd
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        lines = try container.decode([SyncedLine].self, forKey: .lines)
        if let d = try? container.decode(Double.self, forKey: .linesEnd) {
            linesEnd = Int(d * 1000)
        } else {
            let s = try container.decode(String.self, forKey: .linesEnd)
            linesEnd = Int((Double(s) ?? 0) * 1000)
        }
    }
}

struct SyncedLine: Codable, Sendable, Identifiable {
    var time: Int  // ms since start of song
    var text: String
    var translation: String?

    var id: Int { time }

    init(time: Int, text: String, translation: String? = nil) {
        self.time = time
        self.text = text
        self.translation = translation
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        time = Int((try container.decode(Double.self, forKey: .time)) * 1000)
        text = try container.decodeIfPresent(String.self, forKey: .text) ?? ""
        translation = try container.decodeIfPresent(String.self, forKey: .translation)
    }
}

// MARK: - Rich Sync (for future use)

struct SyncedRich: Codable, Sendable {
    var totalTime: Int
    var sections: [SyncedRichSection]
    var agents: [SyncedRichAgent]

    init(totalTime: Int, sections: [SyncedRichSection], agents: [SyncedRichAgent]) {
        self.totalTime = totalTime
        self.sections = sections
        self.agents = agents
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        totalTime = Int((try container.decode(Double.self, forKey: .totalTime)) * 1000)
        sections = try container.decode([SyncedRichSection].self, forKey: .sections)
        agents = try container.decode([SyncedRichAgent].self, forKey: .agents)
    }
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

    init(timeStart: Int, timeEnd: Int, lines: [SyncedRichLine]) {
        self.timeStart = timeStart
        self.timeEnd = timeEnd
        self.lines = lines
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        timeStart = Int((try container.decode(Double.self, forKey: .timeStart)) * 1000)
        timeEnd = Int((try container.decode(Double.self, forKey: .timeEnd)) * 1000)
        lines = try container.decode([SyncedRichLine].self, forKey: .lines)
    }
}

struct SyncedRichLine: Codable, Sendable {
    var timeStart: Int
    var timeEnd: Int
    var text: String
    var segments: [SyncedRichLineSegment]
    var agent: String
    var bgVox: SyncedRichBackgroundLine?

    init(timeStart: Int, timeEnd: Int, text: String, segments: [SyncedRichLineSegment], agent: String, bgVox: SyncedRichBackgroundLine? = nil) {
        self.timeStart = timeStart
        self.timeEnd = timeEnd
        self.text = text
        self.segments = segments
        self.agent = agent
        self.bgVox = bgVox
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        timeStart = Int((try container.decode(Double.self, forKey: .timeStart)) * 1000)
        timeEnd = Int((try container.decode(Double.self, forKey: .timeEnd)) * 1000)
        text = try container.decode(String.self, forKey: .text)
        segments = try container.decode([SyncedRichLineSegment].self, forKey: .segments)
        agent = try container.decode(String.self, forKey: .agent)
        bgVox = try container.decodeIfPresent(SyncedRichBackgroundLine.self, forKey: .bgVox)
    }
}

struct SyncedRichBackgroundLine: Codable, Sendable {
    var timeStart: Int
    var timeEnd: Int
    var text: String
    var segments: [SyncedRichLineSegment]

    init(timeStart: Int, timeEnd: Int, text: String, segments: [SyncedRichLineSegment]) {
        self.timeStart = timeStart
        self.timeEnd = timeEnd
        self.text = text
        self.segments = segments
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        timeStart = Int((try container.decode(Double.self, forKey: .timeStart)) * 1000)
        timeEnd = Int((try container.decode(Double.self, forKey: .timeEnd)) * 1000)
        text = try container.decode(String.self, forKey: .text)
        segments = try container.decode([SyncedRichLineSegment].self, forKey: .segments)
    }
}

struct SyncedRichLineSegment: Codable, Sendable {
    var text: String
    var timeStart: Int
    var timeEnd: Int

    init(text: String, timeStart: Int, timeEnd: Int) {
        self.text = text
        self.timeStart = timeStart
        self.timeEnd = timeEnd
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        text = try container.decode(String.self, forKey: .text)
        timeStart = Int((try container.decode(Double.self, forKey: .timeStart)) * 1000)
        timeEnd = Int((try container.decode(Double.self, forKey: .timeEnd)) * 1000)
    }
}
