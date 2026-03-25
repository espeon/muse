//
//  AlbumCard.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct AlbumCard: View {
    let album: AlbumPartial
    var width: CGFloat = 160

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ArtworkImage(url: album.primaryArtUrl, size: width, cornerRadius: 10)
                .shadow(color: .black.opacity(0.15), radius: 6, x: 0, y: 3)

            VStack(alignment: .leading, spacing: 2) {
                Text(album.name)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .lineLimit(1)
                    .foregroundStyle(.primary)

                if let artist = album.artist {
                    Text(artist.name)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                } else if let year = album.year {
                    Text(String(year))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .frame(width: width)
    }
}

#Preview {
    AlbumCard(album: AlbumPartial(
        id: 1,
        name: "Some Album",
        art: [],
        year: 2024,
        count: 12,
        artist: ArtistPartial(id: 1, name: "Some Artist", picture: nil, numAlbums: nil)
    ))
    .padding()
}
