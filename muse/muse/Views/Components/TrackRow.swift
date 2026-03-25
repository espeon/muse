//
//  TrackRow.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

/// A reusable row for displaying a Track.
/// Used in AlbumDetailView, PlaylistDetailView, and any other track list.
struct TrackRow: View {
    let track: Track
    var trackNumber: Int? = nil
    var isLiked: Bool = false
    /// When provided, the artist name is suppressed if it matches the context artist.
    var albumArtistName: String? = nil
    var isPlaying: Bool = false
    var onLike: (() -> Void)? = nil

    private var showArtist: Bool {
        guard let contextArtist = albumArtistName else { return true }
        return track.displayArtist != contextArtist
    }

    var body: some View {
        HStack(spacing: 12) {
            // Track number / playing indicator
            Group {
                if isPlaying {
                    Image(systemName: "waveform")
                        .font(.caption)
                        .foregroundStyle(.tint)
                } else if let number = trackNumber {
                    Text(String(number))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                } else {
                    Color.clear
                }
            }
            .frame(width: 24, alignment: .trailing)

            // Title + optional artist
            VStack(alignment: .leading, spacing: 2) {
                Text(track.name)
                    .font(.body)
                    .lineLimit(1)
                    .foregroundStyle(isPlaying ? Color.accentColor : .primary)

                if showArtist {
                    Text(track.displayArtist)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }

            Spacer()

            // Badges + duration + like
            HStack(spacing: 8) {
                if track.lossless == true {
                    Text("HI-RES")
                        .font(.system(size: 9, weight: .bold))
                        .foregroundStyle(.white)
                        .padding(.horizontal, 5)
                        .padding(.vertical, 2)
                        .background(Color.accentColor.opacity(0.85), in: RoundedRectangle(cornerRadius: 3))
                }

                Text(track.formattedDuration)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .monospacedDigit()

                if let onLike = onLike {
                    Button(action: onLike) {
                        Image(systemName: isLiked ? "heart.fill" : "heart")
                            .font(.caption)
                            .foregroundStyle(isLiked ? .pink : .secondary)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 10)
        .contentShape(Rectangle())
    }
}
