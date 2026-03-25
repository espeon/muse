//
//  LibraryView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct LibraryView: View {
    @State private var selection = 0

    var body: some View {
        VStack(spacing: 0) {
            Picker("", selection: $selection) {
                Text("Albums").tag(0)
                Text("Artists").tag(1)
                Text("Playlists").tag(2)
            }
            .pickerStyle(.segmented)
            .padding(.horizontal)
            .padding(.vertical, 8)

            Divider()

            switch selection {
            case 0:
                AlbumsView()
            case 1:
                ArtistsView()
            case 2:
                PlaylistsView()
            default:
                AlbumsView()
            }
        }
        .navigationTitle("Library")
        .navigationBarTitleDisplayMode(.large)
    }
}
