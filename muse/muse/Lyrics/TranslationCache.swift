import CryptoKit
import Foundation

enum TranslationCache {
    private static let directory: URL = {
        let caches = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first!
        let dir = caches.appendingPathComponent("translations", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }()

    private struct CacheEntry: Codable {
        let sourceHash: String
        let translations: [String]
    }

    private static func sourceHash(_ texts: [String]) -> String {
        var hasher = SHA256()
        for text in texts {
            hasher.update(data: Data(text.utf8))
            hasher.update(data: Data([0]))
        }
        return hasher.finalize().compactMap { String(format: "%02x", $0) }.joined()
    }

    private static func fileName(trackId: Int, target: String, method: String) -> String {
        "\(trackId)_\(target)_\(method).json"
    }

    static func get(trackId: Int, texts: [String], target: String, method: String) -> [String]? {
        let url = directory.appendingPathComponent(
            fileName(trackId: trackId, target: target, method: method))
        guard let data = try? Data(contentsOf: url),
              let entry = try? JSONDecoder().decode(CacheEntry.self, from: data)
        else { return nil }
        guard entry.sourceHash == sourceHash(texts) else { return nil }
        return entry.translations
    }

    static func set(_ translations: [String], trackId: Int, texts: [String], target: String, method: String) {
        let entry = CacheEntry(sourceHash: sourceHash(texts), translations: translations)
        guard let data = try? JSONEncoder().encode(entry) else { return }
        let url = directory.appendingPathComponent(
            fileName(trackId: trackId, target: target, method: method))
        try? data.write(to: url)
    }
}
