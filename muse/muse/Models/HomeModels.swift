//
//  HomeModels.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import Foundation

enum HomeRowType: String, Codable {
    case album = "Album"
    case artist = "Artist"
    case track = "Track"
}

struct HomeRow: Codable, Identifiable {
    let name: String
    let albums: [AlbumPartial]
    let rowType: HomeRowType
    let resource: String?

    var id: String { name }
}

struct GenreEntry: Codable, Identifiable, Hashable {
    let id: Int
    let name: String
    let albumCount: Int
    let songCount: Int

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: GenreEntry, rhs: GenreEntry) -> Bool {
        lhs.id == rhs.id
    }
}
