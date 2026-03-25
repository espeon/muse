//
//  HomeView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct HomeView: View {
    @Environment(\.apiClient) private var api
    @Environment(PlayerEngine.self) private var playerEngine
    @Environment(AuthManager.self) private var authManager

    @State private var rows: [HomeRow] = []
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var showSettings = false

    var body: some View {
        ScrollView {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity)
                    .padding(.top, 60)
            } else if let error = errorMessage {
                VStack(spacing: 12) {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.largeTitle)
                        .foregroundStyle(.secondary)
                    Text(error)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                    Button("Retry") {
                        Task { await loadHome() }
                    }
                }
                .padding(.top, 60)
                .padding(.horizontal)
            } else {
                LazyVStack(alignment: .leading, spacing: 28) {
                    ForEach(rows) { row in
                        VStack(alignment: .leading, spacing: 12) {
                            Text(row.name)
                                .font(.title2.bold())
                                .padding(.horizontal)

                            ScrollView(.horizontal, showsIndicators: false) {
                                HStack(alignment: .top, spacing: 16) {
                                    ForEach(row.albums) { album in
                                        NavigationLink(value: album) {
                                            AlbumCard(album: album)
                                        }
                                        .buttonStyle(.plain)
                                    }
                                }
                                .padding(.horizontal)
                            }
                        }
                    }
                }
                .padding(.vertical)
            }
        }
        .navigationTitle("Home")
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button(action: { showSettings = true }) {
                    Image(systemName: "gear")
                }
            }
        }
        .navigationDestination(for: AlbumPartial.self) { album in
            AlbumDetailView(album: album)
        }
        .sheet(isPresented: $showSettings) {
            SettingsView()
        }
        .task {
            await loadHome()
        }
        .refreshable {
            await loadHome()
        }
    }

    private func loadHome() async {
        isLoading = true
        errorMessage = nil
        do {
            rows = try await api.fetchHome()
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
