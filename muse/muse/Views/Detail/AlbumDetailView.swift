import SwiftUI

struct AlbumDetailView: View {
    let album: AlbumPartial

    @Environment(\.apiClient) private var api
    @Environment(PlayerEngine.self) private var playerEngine

    @State private var fullAlbum: Album?
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var likedTracks: Set<Int> = []
    @State private var scrollOffset: CGFloat = 0

    var body: some View {
        List {
            header
                .listRowSeparator(.hidden)
                .listRowInsets(EdgeInsets())

            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity)
                    .padding(.top, 40)
                    .listRowSeparator(.hidden)
            } else if let error = errorMessage {
                Text(error)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity)
                    .padding(.top, 40)
                    .listRowSeparator(.hidden)
            } else if let album = fullAlbum, let tracks = album.tracks {
                ForEach(Array(tracks.enumerated()), id: \.element.id) { index, track in
                    TrackRow(
                        track: track,
                        trackNumber: track.number,
                        isLiked: likedTracks.contains(track.id),
                        albumArtistName: album.artist.name,
                        isPlaying: playerEngine.currentTrack?.id == track.id && playerEngine.isPlaying,
                        onLike: { Task { await toggleLike(track: track) } }
                    )
                    .listRowInsets(EdgeInsets())
                    .contentShape(Rectangle())
                    .onTapGesture {
                        Task { await playerEngine.play(tracks: tracks, startingAt: index, apiClient: api) }
                    }
                }
            }
        }
        .listStyle(.plain)
        .onScrollGeometryChange(for: CGFloat.self) { geo in
            geo.contentOffset.y + geo.contentInsets.top
        } action: { _, new in
            scrollOffset = new
        }
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) {
                Text(album.name)
                    .font(.headline)
                    .opacity(min(1.0, max(0.0, (scrollOffset - 300) / 100.0)))
            }
        }
        .task { await loadAlbum() }
    }

    // MARK: - Header

    private var header: some View {
        VStack(spacing: 16) {
            ArtworkImage(url: album.primaryArtUrl, size: 300, cornerRadius: 12)
                .glassEffect(.clear, in: RoundedRectangle(cornerRadius: 12))
                .shadow(color: .black.opacity(0.25), radius: 16, x: 0, y: 8)
                .padding(.top, 12)

            VStack(spacing: 4) {
                Text(album.name)
                    .font(.title2.bold())
                    .multilineTextAlignment(.center)

                if let artist = album.artist {
                    Text(artist.name)
                        .font(.headline)
                        .foregroundStyle(.secondary)
                }

                if let year = album.year {
                    Text(String(year))
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
            }

            if let tracks = fullAlbum?.tracks {
                HStack(spacing: 12) {
                    Button {
                        Task { await playerEngine.play(tracks: tracks, startingAt: 0, apiClient: api) }
                    } label: {
                        Label("Play", systemImage: "play.fill")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.glass)

                    Button {
                        let shuffled = tracks.shuffled()
                        Task { await playerEngine.play(tracks: shuffled, startingAt: 0, apiClient: api) }
                    } label: {
                        Label("Shuffle", systemImage: "shuffle")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.glass)
                }
                .padding(.horizontal)
            }
        }
        .padding(.vertical)
        .frame(maxWidth: .infinity)
    }

    // MARK: - Data

    private func loadAlbum() async {
        isLoading = true
        errorMessage = nil
        do {
            let fetched = try await api.fetchAlbum(id: album.id)
            fullAlbum = fetched
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
            if result.liked { likedTracks.insert(track.id) } else { likedTracks.remove(track.id) }
        } catch {}
    }
}
