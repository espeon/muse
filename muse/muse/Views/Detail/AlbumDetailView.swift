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
    @State private var showAllGenres = false

    init(album: AlbumPartial, preloadedAlbum: Album? = nil) {
        self.album = album
        if let preloadedAlbum {
            _fullAlbum = State(initialValue: preloadedAlbum)
            _isLoading = State(initialValue: false)
            _likedTracks = State(initialValue: Set(preloadedAlbum.tracks?.filter { $0.liked == true }.map(\.id) ?? []))
        }
    }

    var body: some View {
        List {
            header
                .listRowSeparator(.hidden)
                .listRowInsets(EdgeInsets())
                .listRowBackground(Color.clear)
                .background(Color.clear)

            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity)
                    .padding(.top, 40)
                    .listRowSeparator(.hidden)
                    .listRowBackground(Color.clear)
            } else if let error = errorMessage {
                Text(error)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity)
                    .padding(.top, 40)
                    .listRowSeparator(.hidden)
                    .listRowBackground(Color.clear)
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
                    .listRowBackground(Color.clear)
                    .background(Color.clear)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        Task { await playerEngine.play(tracks: tracks, startingAt: index, apiClient: api) }
                    }
                }
            }
        }
        .listStyle(.plain)
        .scrollContentBackground(.hidden)
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
        .navigationDestination(for: ArtistPartial.self) { artist in
            ArtistDetailView(artist: artist)
        }
        .task { await loadAlbum() }
    }

    // MARK: - Header

    private var header: some View {
        VStack(spacing: 16) {
            DynamicArtworkBackground(url: URL(string: album.primaryArtUrl ?? "https://lastfm.freetls.fastly.net/i/u/300x300/32d1f1aaa8e038d36c10eec0dcd20225.jpg"), move: false)
            
            ArtworkImage(url: album.primaryArtUrl, size: 260, cornerRadius: 16)
                .shadow(color: .black.opacity(0.4), radius: 24, x: 0, y: 12)
                .padding(.top, 12)
                .padding(.bottom, 8)
            
            VStack(spacing: 2) {
                Text(album.name)
                    .font(.title2.bold())
                    .multilineTextAlignment(.center)
                
                if let artist = album.artist {
                    NavigationLink(value: artist) {
                        Text(artist.name)
                            .font(.headline)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .center)
                            .multilineTextAlignment(.center)
                    }
                }
                HStack(spacing: 2) {
                    Spacer()
                    HStack(spacing: 4) {
                        if let genres = fullAlbum?.genres, !genres.isEmpty {
                            Text(showAllGenres ? "" : genres[0])
                                .font(.caption)
                            if genres.count > 1 {
                                let remaining = genres.joined(separator: ", ")
                                Text(showAllGenres ? remaining : "•••")
                                    .font(.caption)
                                    .padding(.horizontal, 5)
                                    .padding(.vertical, 2)
                                    .background(
                                        Capsule()
                                            .fill(Color.primary.opacity(0.1))
                                    )
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        withAnimation(.snappy) {
                                            showAllGenres.toggle()
                                        }
                                    }
                            }
                        }
                    }
                    if let genres = fullAlbum?.genres, !genres.isEmpty {
                        Text("⋅")
                    }
                    if let year = album.year {
                        Text(String(year))
                    }
                    if let tracks = fullAlbum?.tracks {
                        if(!tracks.isEmpty && tracks[0].lossless == true) {
                            Text("⋅")
                            Image(systemName: "hifispeaker.fill")
                            Text("Lossless")
                                
                        }
                    }
                    Spacer()
                }
                .padding(.top, 4)
                .font(.caption)
                .foregroundStyle(.secondary)
                
            }
                    
                    
                    if let tracks = fullAlbum?.tracks {
                        HStack(spacing: 12) {
                            Spacer()
                            Button {
                                let shuffled = tracks.shuffled()
                                Task { await playerEngine.play(tracks: shuffled, startingAt: 0, apiClient: api) }
                            } label: {
                                Image(systemName: "shuffle")
                                
                                    .padding(.vertical, 8)
                                    .foregroundStyle(.white)
                            }
                            .buttonStyle(.glassProminent)
                            .tint(.white.opacity(0.2))
                            
                            Button {
                                Task { await playerEngine.play(tracks: tracks, startingAt: 0, apiClient: api) }
                            } label: {
                                HStack(spacing: 8) {
                                    Image(systemName: "play.fill")
                                    Text("Play").fontWeight(.semibold)
                                }
                                .frame(maxWidth: 92)
                                .padding(.horizontal, 16)
                                .padding(.vertical, 6)
                                .foregroundStyle(.black)
                                
                            }
                            .buttonStyle(.glassProminent)
                            .tint(.white)
                            
                            Button {
                                let shuffled = tracks.shuffled()
                                Task { await playerEngine.play(tracks: shuffled, startingAt: 0, apiClient: api) }
                            } label: {
                                Image(systemName: "heart.fill")
                                
                                    .padding(.vertical, 8)
                                    .padding(.horizontal, 1)
                                    .foregroundStyle(.white)
                            }
                            .buttonStyle(.glassProminent)
                            .tint(.white.opacity(0.2))
                            Spacer()
                        }
                        .padding(.horizontal)
                    
                
            }
        }
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

#Preview {
    NavigationStack {
        AlbumDetailView(album: .preview, preloadedAlbum: .preview)
            .environment(PlayerEngine.preview)
    }
}

