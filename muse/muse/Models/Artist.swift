//
//  Artist.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import Foundation

struct ArtistPartial: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let picture: String?
    let numAlbums: Int?

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: ArtistPartial, rhs: ArtistPartial) -> Bool {
        lhs.id == rhs.id
    }
}

struct Artist: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let picture: String?
    let tags: [String]?
    let bio: String?
    let createdAt: Date
    let updatedAt: Date?
    let albums: [AlbumPartial]

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: Artist, rhs: Artist) -> Bool {
        lhs.id == rhs.id
    }
}

struct AllArtistsPartial: Codable {
    let artists: [ArtistPartial]
    let limit: Int
    let cursor: Int
}
