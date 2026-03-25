//
//  PlaylistDetailView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct PlaylistDetailView: View {
    let playlist: PlaylistSummary

    @Environment(\.apiClient) private var api
    @Environment(PlayerEngine.self) private var playerEngine

    @State private var detail: PlaylistDetail?
    @State private var isLoading = true
    @State private var errorMessage: String?

    var body: some View {
        List {
            if let detail {
                // Play button header
                Section {
                    HStack(spacing: 16) {
                        Button {
                            playAll(detail: detail)
                        } label: {
                            Label("Play All", systemImage: "play.fill")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)

                        Button {
                            let shuffled = detail.tracks.shuffled()
                            playPlaylistTracks(shuffled)
                        } label: {
                            Label("Shuffle", systemImage: "shuffle")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)
                    }
                    .listRowInsets(EdgeInsets(top: 12, leading: 16, bottom: 12, trailing: 16))
                }

                // Track list
                Section {
                    ForEach(detail.tracks) { track in
                        PlaylistTrackRow(track: track)
                            .contentShape(Rectangle())
                            .onTapGesture {
                                if let idx = detail.tracks.firstIndex(where: { $0.itemId == track.itemId }) {
                                    playPlaylistTracks(detail.tracks, startingAt: idx)
                                }
                            }
                    }
                    .onDelete { indexSet in
                        deleteItems(at: indexSet, in: detail)
                    }
                    .onMove { from, to in
                        moveItems(from: from, to: to, in: detail)
                    }
                }
            } else if isLoading {
                HStack {
                    Spacer()
                    ProgressView()
                    Spacer()
                }
                .listRowSeparator(.hidden)
            } else if let error = errorMessage {
                Text(error)
                    .foregroundStyle(.secondary)
            }
        }
        .navigationTitle(playlist.name)
        .navigationBarTitleDisplayMode(.large)
        .toolbar {
            if detail != nil {
                EditButton()
            }
        }
        .task {
            await loadDetail()
        }
        .refreshable {
            await loadDetail()
        }
    }

    private func loadDetail() async {
        isLoading = true
        errorMessage = nil
        do {
            detail = try await api.fetchPlaylist(id: playlist.id)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    private func playAll(detail: PlaylistDetail) {
        playPlaylistTracks(detail.tracks, startingAt: 0)
    }

    private func playPlaylistTracks(_ tracks: [PlaylistTrack], startingAt index: Int = 0) {
        // Convert PlaylistTrack to Track-compatible objects via a mapping
        let converted = tracks.map { pt in
            Track(
                id: pt.songId,
                name: pt.name,
                albumArtist: pt.artistName.hashValue,
                artists: [],
                plays: nil,
                duration: pt.duration,
                liked: pt.liked,
                lastPlay: nil,
                year: nil,
                number: pt.number,
                disc: pt.disc,
                lossless: pt.lossless,
                sampleRate: nil,
                bitsPerSample: nil,
                numChannels: nil,
                createdAt: Date(),
                updatedAt: nil,
                album: pt.albumId,
                albumName: pt.albumName,
                artistName: pt.artistName,
                artUrl: pt.artUrl
            )
        }
        Task {
            await playerEngine.play(tracks: converted, startingAt: index, apiClient: api)
        }
    }

    private func deleteItems(at indexSet: IndexSet, in detail: PlaylistDetail) {
        for index in indexSet {
            let track = detail.tracks[index]
            Task {
                try? await api.removeTrackFromPlaylist(playlistId: playlist.id, itemId: track.itemId)
                await loadDetail()
            }
        }
    }

    private func moveItems(from source: IndexSet, to destination: Int, in detail: PlaylistDetail) {
        guard let sourceIndex = source.first else { return }
        let track = detail.tracks[sourceIndex]

        // Determine after which item we are placing this track
        let afterItemId: Int?
        if destination == 0 {
            afterItemId = nil
        } else {
            let adjustedDest = destination > sourceIndex ? destination - 1 : destination
            let after = detail.tracks[adjustedDest]
            afterItemId = after.itemId
        }

        Task {
            try? await api.reorderPlaylistTrack(
                playlistId: playlist.id,
                itemId: track.itemId,
                afterItemId: afterItemId
            )
            await loadDetail()
        }
    }
}

// MARK: - PlaylistTrackRow

private struct PlaylistTrackRow: View {
    let track: PlaylistTrack

    var body: some View {
        HStack(spacing: 12) {
            ArtworkImage(url: track.artUrl, size: 44, cornerRadius: 6)

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(track.name)
                        .font(.body)
                        .lineLimit(1)

                    if track.lossless == true {
                        Text("LOSSLESS")
                            .font(.system(size: 8, weight: .bold))
                            .foregroundStyle(.white)
                            .padding(.horizontal, 4)
                            .padding(.vertical, 2)
                            .background(Color.accentColor, in: RoundedRectangle(cornerRadius: 3))
                    }
                }

                Text(track.artistName)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            Text(track.formattedDuration)
                .font(.caption)
                .foregroundStyle(.tertiary)
                .monospacedDigit()

            if track.liked == true {
                Image(systemName: "heart.fill")
                    .font(.caption)
                    .foregroundStyle(.pink)
            }
        }
        .padding(.vertical, 4)
    }
}
