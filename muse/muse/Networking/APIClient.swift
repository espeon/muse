//
//  APIClient.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import Foundation
import SwiftUI

// MARK: - APIError

enum APIError: LocalizedError {
    case network(Error)
    case unauthorized
    case notFound
    case serverError(String)
    case decodingError(Error)
    case invalidURL

    var errorDescription: String? {
        switch self {
        case .network(let err): return "Network error: \(err.localizedDescription)"
        case .unauthorized: return "Unauthorized. Please log in again."
        case .notFound: return "The requested resource was not found."
        case .serverError(let msg): return "Server error: \(msg)"
        case .decodingError(let err): return "Failed to decode response: \(err.localizedDescription)"
        case .invalidURL: return "Invalid URL."
        }
    }
}

// MARK: - SearchResult

struct SearchResult: Codable, Identifiable {
    let id: Int
    let songName: String
    let artistName: String?
    let albumName: String?
    let picture: String?
}

// MARK: - APIClient

struct APIClient {
    let baseURL: String
    let authToken: () -> String?
    let refreshToken: () async -> Bool

    // MARK: - Decoder

    private var decoder: JSONDecoder {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        let formatterNoFrac = ISO8601DateFormatter()
        formatterNoFrac.formatOptions = [.withInternetDateTime]
        d.dateDecodingStrategy = .custom { decoder in
            let container = try decoder.singleValueContainer()

            // Try decoding as a number (Unix timestamp) first
            if let timestamp = try? container.decode(Double.self) {
                return Date(timeIntervalSince1970: timestamp)
            }

            // Otherwise try string formats (ISO8601/RFC3339)
            let str = try container.decode(String.self)
            if let date = formatter.date(from: str) { return date }
            if let date = formatterNoFrac.date(from: str) { return date }
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Cannot decode date: \(str)")
        }
        return d
    }

    // MARK: - Request Building

    private func makeRequest(
        path: String,
        method: String = "GET",
        queryItems: [URLQueryItem]? = nil,
        body: Data? = nil
    ) throws -> URLRequest {
        guard var components = URLComponents(string: "\(baseURL)\(path)") else {
            throw APIError.invalidURL
        }
        if let queryItems = queryItems {
            components.queryItems = queryItems
        }
        guard let url = components.url else {
            throw APIError.invalidURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.timeoutInterval = 30

        if let token = authToken() {
            request.setValue(token, forHTTPHeaderField: "Authorization")
        }

        if let body = body {
            request.httpBody = body
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        }

        return request
    }

    private func perform<T: Decodable>(_ request: URLRequest, retrying: Bool = true) async throws -> T {
        var currentRequest = request
        let (data, response): (Data, URLResponse)
        do {
            (data, response) = try await URLSession.shared.data(for: currentRequest)
        } catch {
            throw APIError.network(error)
        }

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.network(URLError(.badServerResponse))
        }

        switch httpResponse.statusCode {
        case 200...299:
            break
        case 401:
            if retrying {
                // Try to refresh the token
                let refreshed = await refreshToken()
                if refreshed {
                    // Retry the request with the new token
                    var newRequest = currentRequest
                    if let newToken = authToken() {
                        newRequest.setValue(newToken, forHTTPHeaderField: "Authorization")
                        return try await perform(newRequest, retrying: false)
                    }
                }
                throw APIError.unauthorized
            } else {
                throw APIError.unauthorized
            }
        case 404:
            throw APIError.notFound
        default:
            let message = String(data: data, encoding: .utf8) ?? "Unknown error"
            throw APIError.serverError(message)
        }

        do {
            let decoded = try decoder.decode(T.self, from: data)
            return decoded
        } catch {
            throw APIError.decodingError(error)
        }
    }

    private func performEmpty(_ request: URLRequest) async throws {
        let (_, response): (Data, URLResponse)
        do {
            (_, response) = try await URLSession.shared.data(for: request)
        } catch {
            throw APIError.network(error)
        }

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.network(URLError(.badServerResponse))
        }

        switch httpResponse.statusCode {
        case 200...299:
            return
        case 401:
            throw APIError.unauthorized
        case 404:
            throw APIError.notFound
        default:
            throw APIError.serverError("Status \(httpResponse.statusCode)")
        }
    }

    private func encode<T: Encodable>(_ value: T) throws -> Data {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        return try encoder.encode(value)
    }

    // MARK: - Home

    func fetchHome() async throws -> [HomeRow] {
        let req = try makeRequest(path: "/api/v1/home/")
        return try await perform(req)
    }

    // MARK: - Albums

    func fetchAlbums(cursor: Int = 0, limit: Int = 50, filter: String? = nil, sortby: String = "album", dir: String = "asc") async throws -> AllAlbumsPartial {
        var queryItems = [
            URLQueryItem(name: "cursor", value: String(cursor)),
            URLQueryItem(name: "limit", value: String(limit)),
            URLQueryItem(name: "sortby", value: sortby),
            URLQueryItem(name: "dir", value: dir)
        ]
        if let filter = filter, !filter.isEmpty {
            queryItems.append(URLQueryItem(name: "filter", value: filter))
        }
        let req = try makeRequest(path: "/api/v1/album", queryItems: queryItems)
        return try await perform(req)
    }

    func fetchAlbum(id: Int) async throws -> Album {
        let req = try makeRequest(path: "/api/v1/album/\(id)")
        return try await perform(req)
    }

    // MARK: - Artists

    func fetchArtists(cursor: Int = 0, limit: Int = 50, filter: String? = nil) async throws -> AllArtistsPartial {
        var queryItems = [
            URLQueryItem(name: "cursor", value: String(cursor)),
            URLQueryItem(name: "limit", value: String(limit))
        ]
        if let filter = filter, !filter.isEmpty {
            queryItems.append(URLQueryItem(name: "filter", value: filter))
        }
        let req = try makeRequest(path: "/api/v1/artist", queryItems: queryItems)
        return try await perform(req)
    }

    func fetchArtist(id: Int) async throws -> Artist {
        let req = try makeRequest(path: "/api/v1/artist/\(id)")
        return try await perform(req)
    }

    // MARK: - Tracks

    func fetchTracks(cursor: Int = 0, limit: Int = 50, lossless: Bool? = nil) async throws -> TracksResponse {
        var queryItems = [
            URLQueryItem(name: "cursor", value: String(cursor)),
            URLQueryItem(name: "limit", value: String(limit))
        ]
        if let lossless = lossless {
            queryItems.append(URLQueryItem(name: "lossless", value: lossless ? "true" : "false"))
        }
        let req = try makeRequest(path: "/api/v1/tracks", queryItems: queryItems)
        return try await perform(req)
    }

    func fetchTrack(id: Int) async throws -> Track {
        let req = try makeRequest(path: "/api/v1/track/\(id)")
        return try await perform(req)
    }

    // MARK: - Search

    func searchSongs(query: String) async throws -> [SearchResult] {
        let slug = query.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed) ?? query
        let req = try makeRequest(path: "/api/v1/search/\(slug)")
        return try await perform(req)
    }

    // MARK: - Signing

    func signTrack(id: Int, codec: String? = nil, dps: String? = nil) async throws -> SignResult {
        var queryItems: [URLQueryItem] = []
        if let codec = codec { queryItems.append(URLQueryItem(name: "codec", value: codec)) }
        if let dps = dps { queryItems.append(URLQueryItem(name: "dps", value: dps)) }
        let req = try makeRequest(path: "/api/v1/track/\(id)/sign", queryItems: queryItems.isEmpty ? nil : queryItems)
        return try await perform(req)
    }

    func batchSignTracks(ids: [Int], codec: String? = nil, dps: String? = nil) async throws -> [SignResult] {
        struct BatchSignBody: Encodable {
            let ids: [Int]
            let codec: String?
            let dps: String?
        }
        let body = BatchSignBody(ids: ids, codec: codec, dps: dps)
        let bodyData = try encode(body)
        let req = try makeRequest(path: "/api/v1/tracks/sign", method: "POST", body: bodyData)
        return try await perform(req)
    }

    // MARK: - Track Actions

    func toggleLike(trackId: Int) async throws -> LikedResponse {
        let req = try makeRequest(path: "/api/v1/track/\(trackId)/like", method: "POST")
        return try await perform(req)
    }

    func scrobble(trackId: Int) async throws {
        let req = try makeRequest(path: "/api/v1/track/\(trackId)/scrobble")
        try await performEmpty(req)
    }

    func setPlaying(trackId: Int) async throws {
        let req = try makeRequest(path: "/api/v1/track/\(trackId)/play")
        try await performEmpty(req)
    }

    // MARK: - History

    func fetchHistory(limit: Int = 50, offset: Int = 0) async throws -> [PlayHistoryEntry] {
        let queryItems = [
            URLQueryItem(name: "limit", value: String(limit)),
            URLQueryItem(name: "offset", value: String(offset))
        ]
        let req = try makeRequest(path: "/api/v1/history", queryItems: queryItems)
        return try await perform(req)
    }

    // MARK: - Playlists

    func fetchPlaylists() async throws -> [PlaylistSummary] {
        let req = try makeRequest(path: "/api/v1/playlist")
        return try await perform(req)
    }

    func createPlaylist(name: String, description: String? = nil) async throws -> PlaylistSummary {
        struct CreateBody: Encodable {
            let name: String
            let description: String?
        }
        let body = try encode(CreateBody(name: name, description: description))
        let req = try makeRequest(path: "/api/v1/playlist", method: "POST", body: body)
        return try await perform(req)
    }

    func fetchPlaylist(id: Int) async throws -> PlaylistDetail {
        let req = try makeRequest(path: "/api/v1/playlist/\(id)")
        return try await perform(req)
    }

    func updatePlaylist(id: Int, name: String? = nil, description: String? = nil) async throws -> PlaylistSummary {
        struct UpdateBody: Encodable {
            let name: String?
            let description: String?
        }
        let body = try encode(UpdateBody(name: name, description: description))
        let req = try makeRequest(path: "/api/v1/playlist/\(id)", method: "PUT", body: body)
        return try await perform(req)
    }

    func deletePlaylist(id: Int) async throws {
        let req = try makeRequest(path: "/api/v1/playlist/\(id)", method: "DELETE")
        try await performEmpty(req)
    }

    func addTrackToPlaylist(playlistId: Int, songId: Int) async throws -> PlaylistTrack {
        struct AddTrackBody: Encodable {
            let songId: Int
        }
        let body = try encode(AddTrackBody(songId: songId))
        let req = try makeRequest(path: "/api/v1/playlist/\(playlistId)/tracks", method: "POST", body: body)
        return try await perform(req)
    }

    func removeTrackFromPlaylist(playlistId: Int, itemId: Int) async throws {
        let req = try makeRequest(path: "/api/v1/playlist/\(playlistId)/tracks/\(itemId)", method: "DELETE")
        try await performEmpty(req)
    }

    func reorderPlaylistTrack(playlistId: Int, itemId: Int, afterItemId: Int?) async throws {
        struct ReorderBody: Encodable {
            let afterItemId: Int?
        }
        let body = try encode(ReorderBody(afterItemId: afterItemId))
        let req = try makeRequest(path: "/api/v1/playlist/\(playlistId)/tracks/\(itemId)/position", method: "PUT", body: body)
        try await performEmpty(req)
    }

    // MARK: - Genres

    func fetchGenres() async throws -> [GenreEntry] {
        let req = try makeRequest(path: "/api/v1/genres")
        return try await perform(req)
    }

    // MARK: - Art URL Helper

    func artURL(id: String, width: Int = 400, height: Int = 400) -> String {
        "\(baseURL)/api/v1/art/\(id)?width=\(width)&height=\(height)&format=webp"
    }
}

// MARK: - Environment Key

private struct APIClientKey: EnvironmentKey {
    static let defaultValue = APIClient(baseURL: "", authToken: { nil }, refreshToken: { false })
}

extension EnvironmentValues {
    var apiClient: APIClient {
        get { self[APIClientKey.self] }
        set { self[APIClientKey.self] = newValue }
    }
}
