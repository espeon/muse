//
//  Album.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import Foundation

struct AlbumPartial: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let art: [String]
    let year: Int?
    let count: Int?
    let artist: ArtistPartial?

    var primaryArtUrl: String? {
        art.first
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: AlbumPartial, rhs: AlbumPartial) -> Bool {
        lhs.id == rhs.id
    }
}

struct Album: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let art: [String]
    let year: Int?
    let createdAt: Date
    let updatedAt: Date?
    let artist: ArtistPartial
    let tracks: [Track]?

    var primaryArtUrl: String? {
        art.first
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: Album, rhs: Album) -> Bool {
        lhs.id == rhs.id
    }
}

struct AllAlbumsPartial: Codable {
    let albums: [AlbumPartial]
    let limit: Int
    let offset: Int
}
