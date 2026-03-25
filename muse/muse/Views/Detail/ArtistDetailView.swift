//
//  ArtistDetailView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct ArtistDetailView: View {
    let artist: ArtistPartial

    @Environment(\.apiClient) private var api

    @State private var fullArtist: Artist?
    @State private var isLoading = true
    @State private var errorMessage: String?

    private let columns = [
        GridItem(.flexible(), spacing: 16),
        GridItem(.flexible(), spacing: 16)
    ]

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                // Artist header
                artistHeader

                if isLoading {
                    ProgressView()
                        .padding(.top, 40)
                } else if let error = errorMessage {
                    Text(error)
                        .foregroundStyle(.secondary)
                        .padding(.top, 40)
                } else if let fullArtist {
                    LazyVGrid(columns: columns, spacing: 20) {
                        ForEach(fullArtist.albums) { album in
                            NavigationLink(value: album) {
                                AlbumCard(album: album)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(.horizontal)
                    .padding(.top, 16)
                    .padding(.bottom, 80)
                }
            }
        }
        .navigationTitle(artist.name)
        .navigationBarTitleDisplayMode(.inline)
        .navigationDestination(for: AlbumPartial.self) { album in
            AlbumDetailView(album: album)
        }
        .task {
            await loadArtist()
        }
    }

    private var artistHeader: some View {
        VStack(spacing: 16) {
            if let picture = artist.picture {
                ArtworkImage(url: picture, size: 140, cornerRadius: 70)
                    .shadow(color: .black.opacity(0.2), radius: 8, x: 0, y: 4)
            } else {
                ZStack {
                    Circle()
                        .fill(Color(.systemGray5))
                        .frame(width: 140, height: 140)
                    Image(systemName: "person.fill")
                        .font(.system(size: 56))
                        .foregroundStyle(.secondary)
                }
            }

            VStack(spacing: 6) {
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
            .padding(.horizontal)
        }
        .padding(.top, 24)
        .padding(.bottom, 8)
    }

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
