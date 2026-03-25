//
//  MiniPlayerView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct MiniPlayerView: View {
    @Environment(PlayerEngine.self) private var playerEngine
    var onTap: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            // Album art
            ArtworkImage(url: playerEngine.currentTrack?.artUrl, size: 44, cornerRadius: 6)

            // Track info
            VStack(alignment: .leading, spacing: 2) {
                Text(playerEngine.currentTrack?.name ?? "")
                    .font(.subheadline)
                    .fontWeight(.medium)
                    .lineLimit(1)

                Text(playerEngine.currentTrack?.displayArtist ?? "")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            // Controls
            HStack(spacing: 20) {
                Button {
                    playerEngine.togglePlayPause()
                } label: {
                    Image(systemName: playerEngine.isPlaying ? "pause.fill" : "play.fill")
                        .font(.title3)
                        .foregroundStyle(.primary)
                }
                .buttonStyle(.plain)

                Button {
                    playerEngine.next()
                } label: {
                    Image(systemName: "forward.fill")
                        .font(.title3)
                        .foregroundStyle(.primary)
                }
                .buttonStyle(.plain)
            }
            .padding(.trailing, 4)
        }
        .padding(.horizontal, 16)
        .frame(height: 70)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .padding(.horizontal, 12)
        .contentShape(Rectangle())
        .onTapGesture {
            onTap()
        }
    }
}
