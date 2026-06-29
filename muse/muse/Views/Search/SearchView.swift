//
//  SearchView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct SearchView: View {
    @Environment(\.apiClient) private var apiClient
    @Environment(PlayerEngine.self) private var playerEngine

    @State private var query: String = ""
    @State private var results: [SearchResult] = []
    @State private var isSearching = false
    @State private var errorMessage: String?
    @State private var recentSearches: [String] = []
    @State private var searchTask: Task<Void, Never>?
    @State private var sortBy: SearchSortBy = .relevance
    @State private var sortDir: SortDirection = .ascending

    var body: some View {
        Group {
            if query.isEmpty {
                recentSearchesView
            } else if isSearching {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let error = errorMessage {
                ContentUnavailableView(
                    "Search Failed",
                    systemImage: "magnifyingglass",
                    description: Text(error)
                )
            } else if results.isEmpty {
                ContentUnavailableView.search(text: query)
            } else {
                searchResultsList
            }
        }
        .navigationTitle("Search")
        .searchable(text: $query, prompt: "Search songs")
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Menu {
                    Picker("Sort by", selection: $sortBy) {
                        ForEach(SearchSortBy.allCases, id: \.self) { option in
                            Text(option.label).tag(option)
                        }
                    }
                    Picker("Direction", selection: $sortDir) {
                        ForEach(SortDirection.allCases, id: \.self) { option in
                            Text(option.label).tag(option)
                        }
                    }
                } label: {
                    Label("Sort", systemImage: "arrow.up.arrow.down")
                }
            }
        }
        .onChange(of: query) { _, newQuery in
            searchTask?.cancel()
            if newQuery.isEmpty {
                results = []
                errorMessage = nil
                return
            }
            searchTask = Task {
                try? await Task.sleep(for: .milliseconds(350))
                guard !Task.isCancelled else { return }
                await performSearch(query: newQuery)
            }
        }
        .onChange(of: sortBy) { _, _ in triggerSearch() }
        .onChange(of: sortDir) { _, _ in triggerSearch() }
    }

    // MARK: - Recent Searches

    private var recentSearchesView: some View {
        Group {
            if recentSearches.isEmpty {
                VStack(spacing: 16) {
                    Spacer()
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 48))
                        .foregroundStyle(.secondary)
                    Text("Search your library")
                        .font(.title3)
                        .foregroundStyle(.secondary)
                    Spacer()
                }
            } else {
                List {
                    Section("Recent") {
                        ForEach(recentSearches, id: \.self) { term in
                            Button {
                                query = term
                            } label: {
                                HStack {
                                    Image(systemName: "clock")
                                        .foregroundStyle(.secondary)
                                    Text(term)
                                        .foregroundStyle(.primary)
                                    Spacer()
                                    Image(systemName: "arrow.up.left")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                            }
                        }
                        .onDelete { indexSet in
                            recentSearches.remove(atOffsets: indexSet)
                        }
                    }
                }
            }
        }
    }

    // MARK: - Results

    private var searchResultsList: some View {
        List {
            ForEach(results) { result in
                SearchResultRow(result: result)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        Task { await playSong(result) }
                    }
            }
        }
        .listStyle(.plain)
    }

    // MARK: - Actions

    private func triggerSearch() {
        guard !query.isEmpty else { return }
        searchTask?.cancel()
        searchTask = Task {
            try? await Task.sleep(for: .milliseconds(200))
            guard !Task.isCancelled else { return }
            await performSearch(query: query)
        }
    }

    private func performSearch(query: String) async {
        isSearching = true
        errorMessage = nil
        do {
            results = try await apiClient.searchSongs(query: query, sortby: sortBy, dir: sortDir)
            if !recentSearches.contains(query) {
                recentSearches.insert(query, at: 0)
                if recentSearches.count > 10 {
                    recentSearches = Array(recentSearches.prefix(10))
                }
            }
        } catch {
            if !Task.isCancelled {
                errorMessage = error.localizedDescription
            }
        }
        isSearching = false
    }

    private func playSong(_ result: SearchResult) async {
        // Build a minimal Track and play it; the player will sign the URL
        let track = Track(
            id: result.id,
            name: result.songName,
            albumArtist: 0,
            artists: [],
            plays: nil,
            duration: 0,
            liked: nil,
            lastPlay: nil,
            year: nil,
            number: nil,
            disc: nil,
            lossless: nil,
            sampleRate: nil,
            bitsPerSample: nil,
            numChannels: nil,
            composer: nil,
            isrc: nil,
            bpm: nil,
            createdAt: Date(),
            updatedAt: nil,
            album: 0,
            albumName: result.albumName,
            artistName: result.artistName,
            artUrl: result.picture
        )
        await playerEngine.play(tracks: [track], startingAt: 0, apiClient: apiClient)
    }
}

// MARK: - SearchResultRow

private struct SearchResultRow: View {
    let result: SearchResult

    var body: some View {
        HStack(spacing: 12) {
            ArtworkImage(url: result.picture, size: 44, cornerRadius: 6)

            VStack(alignment: .leading, spacing: 2) {
                Text(result.songName)
                    .font(.body)
                    .lineLimit(1)

                HStack(spacing: 4) {
                    Text(result.artistName)
                    if !result.albumName.isEmpty {
                        Text("•")
                        Text(result.albumName)
                    }
                }
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(1)
            }

            Spacer()
        }
        .padding(.vertical, 2)
    }
}

#Preview {
    SearchView()
        .environment(PlayerEngine.preview)
}
