//
//  Playlist.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import Foundation

struct PlaylistSummary: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let description: String?
    let artPath: String?
    let trackCount: Int
    let createdAt: Date
    let updatedAt: Date?

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: PlaylistSummary, rhs: PlaylistSummary) -> Bool {
        lhs.id == rhs.id
    }
}

struct PlaylistTrack: Codable, Identifiable, Hashable {
    let itemId: Int
    let songId: Int
    let name: String
    let duration: Int
    let number: Int?
    let disc: Int?
    let liked: Bool?
    let lossless: Bool?
    let albumId: Int
    let albumName: String
    let artistName: String
    let artUrl: String?
    let prevItemId: Int?
    let nextItemId: Int?

    var id: Int { itemId }

    var formattedDuration: String {
        let minutes = duration / 60
        let seconds = duration % 60
        return String(format: "%d:%02d", minutes, seconds)
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(itemId)
    }

    static func == (lhs: PlaylistTrack, rhs: PlaylistTrack) -> Bool {
        lhs.itemId == rhs.itemId
    }
}

struct PlaylistDetail: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let description: String?
    let artPath: String?
    let createdAt: Date
    let updatedAt: Date?
    let tracks: [PlaylistTrack]

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: PlaylistDetail, rhs: PlaylistDetail) -> Bool {
        lhs.id == rhs.id
    }
}
