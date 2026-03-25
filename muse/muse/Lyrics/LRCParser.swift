import Foundation

enum LRCParser {
    /// Parse LRC format string into JLF
    /// LRC format: [mm:ss.xx]Line of lyrics
    static func parse(_ lrc: String, source: String = "lrc") -> JLF {
        var lines: [SyncedLine] = []
        var metadata = extractMetadata(from: lrc)

        let pattern = #"\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)$"#
        let regex = try? NSRegularExpression(pattern: pattern, options: .anchorsMatchLines)

        let nsString = lrc as NSString
        let matches =
            regex?.matches(in: lrc, range: NSRange(location: 0, length: nsString.length)) ?? []

        for match in matches {
            guard match.numberOfRanges >= 5 else { continue }

            let minutesStr = nsString.substring(with: match.range(at: 1))
            let secondsStr = nsString.substring(with: match.range(at: 2))
            let centisStr = nsString.substring(with: match.range(at: 3))
            let text = nsString.substring(with: match.range(at: 4)).trimmingCharacters(
                in: .whitespaces)

            guard let minutes = Int(minutesStr),
                let seconds = Int(secondsStr),
                let centis = Int(centisStr)
            else { continue }

            // Convert to milliseconds
            // Handle both 2-digit (centiseconds) and 3-digit (milliseconds) formats
            let ms: Int
            if centisStr.count == 2 {
                ms = (minutes * 60 + seconds) * 1000 + centis * 10
            } else {
                ms = (minutes * 60 + seconds) * 1000 + centis
            }

            // Skip empty lines but keep instrumental breaks as empty
            lines.append(SyncedLine(time: ms, text: text, translation: nil))
        }

        // Sort by time
        lines.sort { $0.time < $1.time }

        // Add intro line if first lyric starts after 5 seconds
        if let firstLine = lines.first, firstLine.time > 5000 {
            lines.insert(SyncedLine(time: 0, text: "", translation: nil), at: 0)
        }

        // Calculate linesEnd (last line time + reasonable duration, or 0 if no lines)
        let linesEnd = lines.last.map { $0.time + 5000 } ?? 0

        return JLF(
            lines: SyncedLines(lines: lines, linesEnd: linesEnd),
            richsync: nil,
            metadata: metadata,
            source: source,
            name: nil,
            message: nil
        )
    }

    /// Extract metadata tags from LRC
    /// Format: [tag:value]
    private static func extractMetadata(from lrc: String) -> SyncedMetadata? {
        var artist: String?
        var title: String?
        var album: String?

        let metaPattern = #"\[(ar|ti|al):([^\]]+)\]"#
        let regex = try? NSRegularExpression(pattern: metaPattern, options: .caseInsensitive)

        let nsString = lrc as NSString
        let matches =
            regex?.matches(in: lrc, range: NSRange(location: 0, length: nsString.length)) ?? []

        for match in matches {
            guard match.numberOfRanges >= 3 else { continue }

            let tag = nsString.substring(with: match.range(at: 1)).lowercased()
            let value = nsString.substring(with: match.range(at: 2)).trimmingCharacters(
                in: .whitespaces)

            switch tag {
            case "ar": artist = value
            case "ti": title = value
            case "al": album = value
            default: break
            }
        }

        // Only return metadata if we have at least artist and title
        guard let artist = artist, let title = title else { return nil }

        return SyncedMetadata(
            artist: artist,
            title: title,
            album: album ?? ""
        )
    }
}
