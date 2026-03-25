//
//  AlbumsView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct AlbumsView: View {
    @Environment(\.apiClient) private var api

    @State private var albums: [AlbumPartial] = []
    @State private var cursor = 0
    @State private var hasMore = true
    @State private var isLoading = false
    @State private var filter = ""
    @State private var searchTask: Task<Void, Never>?

    private let columns = [
        GridItem(.flexible(), spacing: 16),
        GridItem(.flexible(), spacing: 16)
    ]
    private let pageSize = 50

    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 20) {
                ForEach(albums) { album in
                    NavigationLink(value: album) {
                        AlbumCard(album: album)
                    }
                    .buttonStyle(.plain)
                    .onAppear {
                        if album == albums.last && hasMore && !isLoading {
                            Task { await loadMore() }
                        }
                    }
                }

                if isLoading {
                    ProgressView()
                        .frame(maxWidth: .infinity)
                        .gridCellColumns(2)
                        .padding()
                }
            }
            .padding(.horizontal)
            .padding(.top, 12)
        }
        .searchable(text: $filter, prompt: "Search albums")
        .navigationDestination(for: AlbumPartial.self) { album in
            AlbumDetailView(album: album)
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
            if albums.isEmpty {
                await resetAndLoad()
            }
        }
    }

    private func resetAndLoad() async {
        cursor = 0
        hasMore = true
        albums = []
        await loadMore()
    }

    private func loadMore() async {
        guard hasMore, !isLoading else { return }
        isLoading = true
        do {
            let result = try await api.fetchAlbums(
                cursor: cursor,
                limit: pageSize,
                filter: filter.isEmpty ? nil : filter
            )
            albums.append(contentsOf: result.albums)
            cursor = result.offset + result.albums.count
            hasMore = result.albums.count == pageSize
        } catch {
            // silently ignore; user can scroll to retry via onAppear
        }
        isLoading = false
    }
}
