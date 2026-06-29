//
//  ArtistsView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct ArtistsView: View {
    @Environment(\.apiClient) private var api

    @State private var artists: [ArtistPartial] = []
    @State private var cursor = 0
    @State private var hasMore = true
    @State private var isLoading = false
    @State private var filter = ""
    @State private var searchTask: Task<Void, Never>?
    @State private var errorMessage: String?

    private let pageSize = 50

    var body: some View {
        List {
            ForEach(artists) { artist in
                NavigationLink(value: artist) {
                    HStack(spacing: 12) {
                        ArtworkImage(url: artist.picture, size: 44, cornerRadius: 22)

                        VStack(alignment: .leading, spacing: 2) {
                            Text(artist.name)
                                .font(.body)
                            if let count = artist.numAlbums {
                                Text("\(count) \(count == 1 ? "album" : "albums")")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                    .padding(.vertical, 4)
                }
                .onAppear {
                    if artist == artists.last && hasMore && !isLoading {
                        Task { await loadMore() }
                    }
                }
            }

            if isLoading {
                HStack {
                    Spacer()
                    ProgressView()
                    Spacer()
                }
                .listRowSeparator(.hidden)
            }

            if let error = errorMessage, !isLoading {
                VStack(spacing: 8) {
                    Text(error)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                    Button("Retry") {
                        Task { await resetAndLoad() }
                    }
                    .font(.subheadline)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 40)
                .listRowSeparator(.hidden)
                .listRowBackground(Color.clear)
            }
        }
        .listStyle(.plain)
        .searchable(text: $filter, prompt: "Search artists")
        .navigationDestination(for: ArtistPartial.self) { artist in
            ArtistDetailView(artist: artist)
        }
        .onChange(of: filter) { _, _ in
            searchTask?.cancel()
            searchTask = Task {
                try? await Task.sleep(for: .milliseconds(300))
                guard !Task.isCancelled else { return }
                await resetAndLoad()
            }
        }
        .task {
            if artists.isEmpty {
                await resetAndLoad()
            }
        }
    }

    private func resetAndLoad() async {
        cursor = 0
        hasMore = true
        artists = []
        errorMessage = nil
        await loadMore()
    }

    private func loadMore() async {
        guard hasMore, !isLoading else { return }
        isLoading = true
        do {
            let result = try await api.fetchArtists(
                cursor: cursor,
                limit: pageSize,
                filter: filter.isEmpty ? nil : filter
            )
            artists.append(contentsOf: result.artists)
            cursor = result.cursor
            hasMore = result.artists.count == pageSize
            errorMessage = nil
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}

#Preview {
    ArtistsView()
}
