//
//  Track.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import Foundation

struct Track: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let albumArtist: Int
    let artists: [ArtistPartial]
    let plays: Int?
    let duration: Int
    let liked: Bool?
    let lastPlay: Date?
    let year: Int?
    let number: Int?
    let disc: Int?
    let lossless: Bool?
    let sampleRate: Int?
    let bitsPerSample: Int?
    let numChannels: Int?
    let createdAt: Date
    let updatedAt: Date?
    let album: Int
    let albumName: String
    let artistName: String
    let artUrl: String?

    var formattedDuration: String {
        let minutes = duration / 60
        let seconds = duration % 60
        return String(format: "%d:%02d", minutes, seconds)
    }

    var displayArtist: String {
        artistName
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: Track, rhs: Track) -> Bool {
        lhs.id == rhs.id
    }
}

struct TrackListItem: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let duration: Int
    let number: Int?
    let disc: Int?
    let lossless: Bool?
    let sampleRate: Int?
    let bitsPerSample: Int?
    let numChannels: Int?
    let albumId: Int
    let albumName: String
    let artistId: Int
    let artistName: String
    let artUrl: String?
    let liked: Bool?

    var formattedDuration: String {
        let minutes = duration / 60
        let seconds = duration % 60
        return String(format: "%d:%02d", minutes, seconds)
    }

    var displayArtist: String {
        artistName
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: TrackListItem, rhs: TrackListItem) -> Bool {
        lhs.id == rhs.id
    }
}

struct TracksResponse: Codable {
    let tracks: [TrackListItem]
    let total: Int
    let limit: Int
    let cursor: Int
}

struct SignResult: Codable, Identifiable {
    let id: Int
    let url: String
    let signedAt: Date
    let expiresAt: Date
}

struct LikedResponse: Codable {
    let liked: Bool
}

struct PlayHistoryEntry: Codable, Identifiable {
    let playedAt: Date
    let songId: Int
    let name: String
    let duration: Int
    let albumId: Int
    let albumName: String
    let artistId: Int
    let artistName: String
    let liked: Bool

    var id: Int { songId }

    var formattedDuration: String {
        let minutes = duration / 60
        let seconds = duration % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}
