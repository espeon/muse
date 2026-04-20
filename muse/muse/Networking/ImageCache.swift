import SwiftUI

actor ImageCache {
    static let shared = ImageCache()

    private let cache = NSCache<NSString, UIImage>()
    private var inFlightRequests: [URL: Task<UIImage?, Never>] = [:]

    private let urlSession: URLSession

    private init() {
        cache.countLimit = 200
        cache.totalCostLimit = 100 * 1024 * 1024

        let configuration = URLSessionConfiguration.default
        configuration.requestCachePolicy = .returnCacheDataElseLoad
        configuration.urlCache = URLCache(
            memoryCapacity: 50 * 1024 * 1024,
            diskCapacity: 200 * 1024 * 1024
        )
        urlSession = URLSession(configuration: configuration)
    }

    func clearCache() {
        cache.removeAllObjects()
        urlSession.configuration.urlCache?.removeAllCachedResponses()
    }

    func isCached(url: URL?) -> Bool {
        guard let url = url else { return false }
        return cache.object(forKey: url.absoluteString as NSString) != nil
    }

    func preload(url: URL?) {
        guard let url = url else { return }
        let key = url.absoluteString as NSString
        if cache.object(forKey: key) != nil || inFlightRequests[url] != nil {
            return
        }
        Task {
            _ = await image(for: url)
        }
    }

    func image(for url: URL?) async -> UIImage? {
        guard let url = url else { return nil }
        let key = url.absoluteString as NSString

        if let cached = cache.object(forKey: key) {
            return cached
        }

        if let existing = inFlightRequests[url] {
            return await existing.value
        }

        let task = Task<UIImage?, Never> {
            guard let (data, _) = try? await urlSession.data(from: url),
                  let image = UIImage(data: data) else {
                return nil
            }
            cache.setObject(image, forKey: key, cost: data.count)
            return image
        }

        inFlightRequests[url] = task
        let result = await task.value
        inFlightRequests[url] = nil
        return result
    }
}
