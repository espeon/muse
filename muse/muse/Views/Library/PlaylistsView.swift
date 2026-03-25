//
//  PlaylistsView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct PlaylistsView: View {
    @Environment(\.apiClient) private var api

    @State private var playlists: [PlaylistSummary] = []
    @State private var isLoading = false
    @State private var showNewPlaylist = false
    @State private var newPlaylistName = ""
    @State private var isCreating = false

    var body: some View {
        List {
            ForEach(playlists) { playlist in
                NavigationLink(value: playlist) {
                    HStack(spacing: 12) {
                        ArtworkImage(url: playlist.artPath, size: 44, cornerRadius: 6)

                        VStack(alignment: .leading, spacing: 2) {
                            Text(playlist.name)
                                .font(.body)
                            Text("\(playlist.trackCount) \(playlist.trackCount == 1 ? "song" : "songs")")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .padding(.vertical, 4)
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
        }
        .listStyle(.plain)
        .navigationDestination(for: PlaylistSummary.self) { playlist in
            PlaylistDetailView(playlist: playlist)
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button {
                    newPlaylistName = ""
                    showNewPlaylist = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showNewPlaylist) {
            NavigationStack {
                Form {
                    Section("Playlist Name") {
                        TextField("My Playlist", text: $newPlaylistName)
                    }
                }
                .navigationTitle("New Playlist")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("Cancel") {
                            showNewPlaylist = false
                        }
                    }
                    ToolbarItem(placement: .confirmationAction) {
                        Button("Create") {
                            createPlaylist()
                        }
                        .disabled(newPlaylistName.trimmingCharacters(in: .whitespaces).isEmpty || isCreating)
                    }
                }
            }
            .presentationDetents([.medium])
        }
        .task {
            await loadPlaylists()
        }
        .refreshable {
            await loadPlaylists()
        }
    }

    private func loadPlaylists() async {
        isLoading = true
        do {
            playlists = try await api.fetchPlaylists()
        } catch {
            // silently ignore
        }
        isLoading = false
    }

    private func createPlaylist() {
        let name = newPlaylistName.trimmingCharacters(in: .whitespaces)
        guard !name.isEmpty else { return }
        isCreating = true
        Task {
            do {
                let created = try await api.createPlaylist(name: name)
                playlists.insert(created, at: 0)
                showNewPlaylist = false
            } catch {
                // silently ignore
            }
            isCreating = false
        }
    }
}
