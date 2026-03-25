//
//  AlbumDetailView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct AlbumDetailView: View {
    let album: AlbumPartial

    @Environment(\.apiClient) private var api
    @Environment(PlayerEngine.self) private var playerEngine

    @State private var fullAlbum: Album?
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var likedTracks: Set<Int> = []

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                // Header
                albumHeader

                if isLoading {
                    ProgressView()
                        .padding(.top, 40)
                } else if let error = errorMessage {
                    Text(error)
                        .foregroundStyle(.secondary)
                        .padding(.top, 40)
                } else if let album = fullAlbum, let tracks = album.tracks {
                    // Play / Shuffle buttons
                    HStack(spacing: 16) {
                        Button {
                            Task {
                                await playerEngine.play(tracks: tracks, startingAt: 0, apiClient: api)
                            }
                        } label: {
                            Label("Play", systemImage: "play.fill")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)

                        Button {
                            let shuffled = tracks.shuffled()
                            Task {
                                await playerEngine.play(tracks: shuffled, startingAt: 0, apiClient: api)
                            }
                        } label: {
                            Label("Shuffle", systemImage: "shuffle")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)
                    }
                    .padding(.horizontal)
                    .padding(.top, 20)
                    .padding(.bottom, 12)

                    Divider()
                        .padding(.horizontal)

                    // Track list
                    LazyVStack(spacing: 0) {
                        ForEach(Array(tracks.enumerated()), id: \.element.id) { index, track in
                            TrackRow(
                                track: track,
                                trackNumber: track.number,
                                isLiked: likedTracks.contains(track.id),
                                albumArtistName: album.artist.name,
                                onLike: {
                                    Task { await toggleLike(track: track) }
                                }
                            )
                            .contentShape(Rectangle())
                            .onTapGesture {
                                Task {
                                    await playerEngine.play(tracks: tracks, startingAt: index, apiClient: api)
                                }
                            }

                            if index < tracks.count - 1 {
                                Divider()
                                    .padding(.leading, 56)
                            }
                        }
                    }
                    .padding(.bottom, 80)
                }
            }
        }
        .navigationBarTitleDisplayMode(.inline)
        .task {
            await loadAlbum()
        }
    }

    private var albumHeader: some View {
        VStack(spacing: 12) {
            GeometryReader { geo in
                ArtworkImage(url: album.primaryArtUrl, size: geo.size.width, cornerRadius: 0)
            }
            .aspectRatio(1, contentMode: .fit)

            VStack(spacing: 4) {
                Text(album.name)
                    .font(.title2.bold())
                    .multilineTextAlignment(.center)

                if let artist = album.artist {
                    Text(artist.name)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                if let year = album.year {
                    Text(String(year))
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
            }
            .padding(.horizontal)
        }
    }

    private func loadAlbum() async {
        isLoading = true
        errorMessage = nil
        do {
            let fetched = try await api.fetchAlbum(id: album.id)
            fullAlbum = fetched
            // Seed liked state from track data
            if let tracks = fetched.tracks {
                for track in tracks where track.liked == true {
                    likedTracks.insert(track.id)
                }
            }
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    private func toggleLike(track: Track) async {
        do {
            let result = try await api.toggleLike(trackId: track.id)
            if result.liked {
                likedTracks.insert(track.id)
            } else {
                likedTracks.remove(track.id)
            }
        } catch {
            // silently ignore
        }
    }
}
