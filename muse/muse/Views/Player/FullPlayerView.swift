//
//  FullPlayerView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct FullPlayerView: View {
    @Environment(PlayerEngine.self) private var playerEngine
    @Environment(\.apiClient) private var apiClient
    @Environment(\.dismiss) private var dismiss

    @State private var isDraggingSlider = false
    @State private var sliderValue: Double = 0
    @State private var showQueue = false
    @State private var likedState: Bool? = nil
    @State private var isTogglingLike = false

    var body: some View {
        NavigationStack {
            ZStack {
                // Background: blurred artwork
                if let artUrl = playerEngine.currentTrack?.artUrl {
                    AsyncImage(url: URL(string: artUrl)) { phase in
                        if case .success(let image) = phase {
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                                .ignoresSafeArea()
                                .blur(radius: 60)
                                .opacity(0.4)
                        }
                    }
                }

                Rectangle()
                    .fill(.ultraThinMaterial)
                    .ignoresSafeArea()

                ScrollView {
                    VStack(spacing: 32) {
                        // Large album art
                        ArtworkImage(
                            url: playerEngine.currentTrack?.artUrl,
                            size: 300,
                            cornerRadius: 16
                        )
                        .shadow(color: .black.opacity(0.3), radius: 20, x: 0, y: 10)
                        .scaleEffect(playerEngine.isPlaying ? 1.0 : 0.92)
                        .animation(.spring(duration: 0.4), value: playerEngine.isPlaying)

                        // Track info + like
                        HStack(alignment: .center) {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(playerEngine.currentTrack?.name ?? "")
                                    .font(.title2)
                                    .fontWeight(.bold)
                                    .lineLimit(2)

                                Text(playerEngine.currentTrack?.displayArtist ?? "")
                                    .font(.body)
                                    .foregroundStyle(.secondary)
                                    .lineLimit(1)
                            }

                            Spacer()

                            Button {
                                toggleLike()
                            } label: {
                                Image(systemName: currentLiked ? "heart.fill" : "heart")
                                    .font(.title2)
                                    .foregroundStyle(currentLiked ? .pink : .secondary)
                            }
                            .disabled(isTogglingLike)
                        }
                        .padding(.horizontal, 4)

                        // Progress bar
                        VStack(spacing: 6) {
                            Slider(
                                value: Binding(
                                    get: { isDraggingSlider ? sliderValue : playerEngine.currentTime },
                                    set: { newValue in
                                        sliderValue = newValue
                                        isDraggingSlider = true
                                    }
                                ),
                                in: 0...max(playerEngine.duration, 1)
                            ) { editing in
                                isDraggingSlider = editing
                                if !editing {
                                    playerEngine.seek(to: sliderValue)
                                }
                            }
                            .tint(.primary)

                            HStack {
                                Text(formatTime(isDraggingSlider ? sliderValue : playerEngine.currentTime))
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .monospacedDigit()

                                Spacer()

                                Text("-\(formatTime(max(0, playerEngine.duration - (isDraggingSlider ? sliderValue : playerEngine.currentTime))))")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .monospacedDigit()
                            }
                        }

                        // Playback controls
                        HStack(spacing: 44) {
                            Button {
                                playerEngine.previous()
                            } label: {
                                Image(systemName: "backward.fill")
                                    .font(.title)
                                    .foregroundStyle(.primary)
                            }

                            Button {
                                playerEngine.togglePlayPause()
                            } label: {
                                Image(systemName: playerEngine.isPlaying ? "pause.circle.fill" : "play.circle.fill")
                                    .font(.system(size: 72))
                                    .foregroundStyle(.primary)
                            }

                            Button {
                                playerEngine.next()
                            } label: {
                                Image(systemName: "forward.fill")
                                    .font(.title)
                                    .foregroundStyle(.primary)
                            }
                        }

                        // Volume
                        HStack(spacing: 12) {
                            Image(systemName: "speaker.fill")
                                .foregroundStyle(.secondary)
                                .font(.caption)

                            Slider(
                                value: Binding(
                                    get: { Double(playerEngine.volume) },
                                    set: { playerEngine.setVolume(Float($0)) }
                                ),
                                in: 0...1
                            )
                            .tint(.secondary)

                            Image(systemName: "speaker.wave.3.fill")
                                .foregroundStyle(.secondary)
                                .font(.caption)
                        }

                        // Queue info
                        Button {
                            showQueue = true
                        } label: {
                            HStack {
                                Image(systemName: "list.bullet")
                                Text("Queue")
                                    .fontWeight(.medium)
                            }
                            .foregroundStyle(.secondary)
                        }

                        // Lossless badge if applicable
                        if playerEngine.currentTrack?.lossless == true {
                            HStack(spacing: 6) {
                                if let sampleRate = playerEngine.currentTrack?.sampleRate,
                                   let bits = playerEngine.currentTrack?.bitsPerSample {
                                    Text("LOSSLESS \(bits)-bit / \(sampleRate / 1000) kHz")
                                        .font(.caption2)
                                        .fontWeight(.semibold)
                                        .foregroundStyle(.tint)
                                }
                            }
                        }
                    }
                    .padding(24)
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        dismiss()
                    } label: {
                        Image(systemName: "chevron.down")
                            .fontWeight(.semibold)
                    }
                }

                ToolbarItem(placement: .principal) {
                    VStack(spacing: 2) {
                        Text("Now Playing")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                        Text(playerEngine.currentTrack?.albumName ?? "")
                            .font(.caption)
                            .fontWeight(.medium)
                            .lineLimit(1)
                    }
                }
            }
        }
        .sheet(isPresented: $showQueue) {
            QueueView()
        }
        .onChange(of: playerEngine.currentTrack?.id) { _, newId in
            likedState = playerEngine.currentTrack?.liked
        }
        .onAppear {
            likedState = playerEngine.currentTrack?.liked
        }
    }

    // MARK: - Helpers

    private var currentLiked: Bool {
        likedState ?? playerEngine.currentTrack?.liked ?? false
    }

    private func formatTime(_ seconds: Double) -> String {
        let s = Int(seconds)
        return String(format: "%d:%02d", s / 60, s % 60)
    }

    private func toggleLike() {
        guard let track = playerEngine.currentTrack, !isTogglingLike else { return }
        isTogglingLike = true
        let newLiked = !(likedState ?? track.liked ?? false)
        likedState = newLiked

        Task {
            do {
                let result = try await apiClient.toggleLike(trackId: track.id)
                await MainActor.run {
                    likedState = result.liked
                }
            } catch {
                await MainActor.run {
                    likedState = track.liked
                }
            }
            await MainActor.run {
                isTogglingLike = false
            }
        }
    }
}

// MARK: - QueueView

private struct QueueView: View {
    @Environment(PlayerEngine.self) private var playerEngine
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                ForEach(Array(playerEngine.queue.enumerated()), id: \.element.id) { index, track in
                    HStack(spacing: 12) {
                        ArtworkImage(url: track.artUrl, size: 44, cornerRadius: 6)

                        VStack(alignment: .leading, spacing: 2) {
                            Text(track.name)
                                .font(.subheadline)
                                .lineLimit(1)
                                .foregroundStyle(index == playerEngine.currentIndex ? Color.accentColor : .primary)
                            Text(track.displayArtist)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                        }

                        Spacer()

                        if index == playerEngine.currentIndex {
                            Image(systemName: "waveform")
                                .foregroundStyle(.tint)
                                .font(.caption)
                        }
                    }
                    .padding(.vertical, 2)
                }
            }
            .navigationTitle("Queue")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }
}
