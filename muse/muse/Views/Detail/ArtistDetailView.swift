import SwiftUI

struct ArtistDetailView: View {
    let artist: ArtistPartial

    @Environment(\.apiClient) private var api

    @State private var fullArtist: Artist?
    @State private var isLoading = true
    @State private var errorMessage: String?
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
            } else if let fullArtist, !fullArtist.albums.isEmpty {
                Section("Albums") {
                    ForEach(fullArtist.albums) { album in
                        NavigationLink(value: album) {
                            albumRow(album)
                        }
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
                Text(artist.name)
                    .font(.headline)
                    .opacity(min(1.0, max(0.0, (scrollOffset - 120) / 60.0)))
            }
        }
        .navigationDestination(for: AlbumPartial.self) { album in
            AlbumDetailView(album: album)
        }
        .task { await loadArtist() }
    }

    // MARK: - Header

    private var header: some View {
        VStack(spacing: 16) {
            if let picture = fullArtist?.picture ?? artist.picture {
                ArtworkImage(url: picture, size: 150, cornerRadius: 75)
                    .shadow(color: .black.opacity(0.2), radius: 8, x: 0, y: 4)
            } else {
                ZStack {
                    Circle()
                        .fill(Color(.systemGray5))
                        .frame(width: 150, height: 150)
                    Image(systemName: "person.fill")
                        .font(.system(size: 60))
                        .foregroundStyle(.secondary)
                }
            }

            Text(artist.name)
                .font(.title.bold())
                .multilineTextAlignment(.center)

            if let bio = fullArtist?.bio, !bio.isEmpty {
                Text(bio)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .lineLimit(4)
                    .padding(.horizontal)
            }

            if let tags = fullArtist?.tags, !tags.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(tags, id: \.self) { tag in
                            Text(tag)
                                .font(.caption)
                                .padding(.horizontal, 10)
                                .padding(.vertical, 4)
                                .background(Color(.secondarySystemBackground))
                                .clipShape(Capsule())
                        }
                    }
                    .padding(.horizontal)
                }
            }
        }
        .padding(.top, 24)
        .padding(.bottom, 16)
        .frame(maxWidth: .infinity)
    }

    // MARK: - Album row

    private func albumRow(_ album: AlbumPartial) -> some View {
        HStack(spacing: 12) {
            ArtworkImage(url: album.primaryArtUrl, size: 52, cornerRadius: 6)
            VStack(alignment: .leading, spacing: 2) {
                Text(album.name)
                    .font(.body)
                    .lineLimit(1)
                if let year = album.year {
                    Text(String(year))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding(.vertical, 2)
    }

    // MARK: - Data

    private func loadArtist() async {
        isLoading = true
        errorMessage = nil
        do {
            fullArtist = try await api.fetchArtist(id: artist.id)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
