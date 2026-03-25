import SwiftUI

struct NowPlayingSheetView: View {
    @Bindable var player: PlayerEngine
    var dismiss: () -> Void

    @State private var lyrics: JLF?
    @State private var showLyrics: Bool = false
    @State private var dragOffset: CGSize = .zero
    @State private var isDraggingVolume: Bool = false
    @State private var localVolume: Float?

    @Environment(\.umiClient) private var umiClient

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                // Background
                if let track = player.currentTrack,
                    let artUrl = track.artUrl,
                    let url = URL(string: artUrl)
                {
                    AsyncImage(url: url) { image in
                        image
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .blur(radius: 60)
                            .opacity(0.5)
                    } placeholder: {
                        Color.black
                    }
                    .ignoresSafeArea()
                } else {
                    Color.black.ignoresSafeArea()
                }

                VStack(spacing: 20) {
                    // Grab handle
                    Capsule()
                        .fill(Color.white.opacity(0.3))
                        .frame(width: 40, height: 4)
                        .padding(.top, 10)

                    if showLyrics, let lyrics = lyrics {
                        // Lyrics View
                        SyncedLyricsView(
                            lyrics: lyrics,
                            currentTimeMs: Int(player.currentTime * 1000),
                            onSeek: { ms in
                                player.seek(to: Double(ms) / 1000.0)
                            }
                        )
                        .transition(.move(edge: .bottom).combined(with: .opacity))
                    } else {
                        // Artwork
                        AnimatedArtworkCard(
                            url: player.currentTrack.flatMap { $0.artUrl }.flatMap {
                                URL(string: $0)
                            },
                            queueItemId: player.currentTrack?.id.description,
                            queueIndex: player.currentIndex,
                            size: geometry.size.width - 60,
                            cornerRadius: 12,
                            playingScale: player.isPlaying ? 1.0 : 0.9
                        )
                        .frame(height: geometry.size.width - 60)

                        // Info
                        VStack(spacing: 4) {
                            Text(player.currentTrack?.name ?? "Unknown Title")
                                .font(.title2)
                                .fontWeight(.bold)
                                .lineLimit(1)
                                .multilineTextAlignment(.center)

                            Text(player.currentTrack?.displayArtist ?? "Unknown Artist")
                                .font(.title3)
                                .foregroundColor(.secondary)
                                .lineLimit(1)
                                .multilineTextAlignment(.center)
                        }
                        .padding(.horizontal)

                        // Progress
                        VStack(spacing: 8) {
                            Slider(
                                value: Binding(
                                    get: { player.currentTime },
                                    set: { player.seek(to: $0) }
                                ),
                                in: 0...(player.duration > 0 ? player.duration : 1)
                            )
                            .tint(.white)

                            HStack {
                                Text(formatTime(player.currentTime))
                                Spacer()
                                Text(formatTime(player.duration))
                            }
                            .font(.caption)
                            .foregroundColor(.secondary)
                        }
                        .padding(.horizontal, 30)

                        // Controls
                        HStack(spacing: 50) {
                            Button {
                                player.previous()
                            } label: {
                                Image(systemName: "backward.fill")
                                    .font(.title)
                            }

                            Button {
                                player.togglePlayPause()
                            } label: {
                                Image(
                                    systemName: player.isPlaying
                                        ? "pause.circle.fill" : "play.circle.fill"
                                )
                                .font(.system(size: 64))
                            }

                            Button {
                                player.next()
                            } label: {
                                Image(systemName: "forward.fill")
                                    .font(.title)
                            }
                        }
                        .foregroundColor(.white)

                        // Volume
                        HStack {
                            Image(systemName: "speaker.fill")
                            Slider(
                                value: Binding(
                                    get: { localVolume ?? player.volume },
                                    set: { val in
                                        localVolume = val
                                        player.setVolume(val)
                                    }
                                )
                            )
                            .tint(.white)
                            Image(systemName: "speaker.wave.3.fill")
                        }
                        .padding(.horizontal, 30)
                        .padding(.top, 20)

                        // Lyrics Toggle
                        Button {
                            withAnimation {
                                showLyrics.toggle()
                            }
                        } label: {
                            Image(systemName: "quote.bubble")
                                .font(.title2)
                                .foregroundColor(showLyrics ? .white : .white.opacity(0.5))
                        }
                        .padding(.bottom, 30)
                    }
                }
            }
        }
        .onAppear {
            loadLyrics()
        }
        .onChange(of: player.currentTrack) { _, _ in
            loadLyrics()
        }
    }

    private func formatTime(_ seconds: Double) -> String {
        let m = Int(seconds) / 60
        let s = Int(seconds) % 60
        return String(format: "%d:%02d", m, s)
    }

    private func loadLyrics() {
        guard let track = player.currentTrack else { return }

        Task {
            do {
                let jlf = try await umiClient.fetchLyrics(
                    track: track.name,
                    artist: track.displayArtist,
                    album: track.albumName
                )
                await MainActor.run {
                    self.lyrics = jlf
                }
            } catch {
                print("Failed to fetch lyrics: \(error)")
                await MainActor.run {
                    self.lyrics = nil
                }
            }
        }
    }
}
