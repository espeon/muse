import SwiftUI
import Translation

struct NowPlayingSheetView: View {
    @Bindable var player: PlayerEngine
    var dismiss: () -> Void

    @State private var showLyrics = false
    @State private var lyrics: JLF?
    @State private var isDraggingProgress = false
    @State private var progressDragValue: Double = 0
    @State private var likedState: Bool? = nil
    @State private var isTogglingLike = false
    @State private var showQueue = false

    @State private var translations: [String]? = nil
    @State private var bgVoxTranslations: [String]? = nil
    @State private var showTranslationPicker = false
    @State private var selectedTranslationLanguage: String? = nil
    @State private var translationConfig: TranslationSession.Configuration? = nil
    @StateObject private var translationService = TranslationService()

    @Environment(\.umiClient) private var umiClient
    @Environment(\.apiClient) private var apiClient

    var body: some View {
        GeometryReader { geo in
            ZStack {
                artworkBackground

                VStack(spacing: 0) {

                    Button(action: dismiss) {
                        RoundedRectangle(cornerRadius: 999)
                            .fill(Color.gray.opacity(0.45))
                            .frame(maxWidth: 60, maxHeight: 7.5)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.top, 48)
                    .safeAreaPadding(.top)

                    Spacer().frame(minHeight: 8)

                    ZStack {
                        artworkContent(geo: geo)
                            .opacity(showLyrics ? 0 : 1)
                            .scaleEffect(showLyrics ? 0.8 : 1)

                        if showLyrics {
                            lyricsView
                                .transition(.opacity.combined(with: .scale(scale: 0.95)))
                        }

                        if showLyrics {
                            VStack {
                                trackInfo(compact: true)
                                    .padding(.top, 24)
                                    .padding(.horizontal, 10)
                                    .safeAreaPadding(.top)
                                Spacer()
                            }
                            .transition(
                                .asymmetric(
                                    insertion: .move(edge: .top).combined(with: .opacity),
                                    removal: .move(edge: .top).combined(with: .opacity)
                                ))
                        }
                    }
                    .frame(height: showLyrics ? geo.size.height * 0.72 : nil)
                    .animation(.spring(response: 0.4, dampingFraction: 0.8), value: showLyrics)

                    Spacer().frame(minHeight: 8, maxHeight: 24)

                    controls(compact: showLyrics)
                        .padding(.top, showLyrics ? 6 : 10)
                        .padding(.bottom, showLyrics ? 80 : 25)
                        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: showLyrics)
                }
                .padding(.top, geo.safeAreaInsets.top)
                .padding(.bottom, geo.safeAreaInsets.bottom)
                .frame(width: geo.size.width, height: geo.size.height)
            }
            .frame(width: geo.size.width, height: geo.size.height)
            .clipped()
        }
        .ignoresSafeArea()
        .sheet(isPresented: $showQueue) {
            NowPlayingQueueSheet(player: player)
        }
        .sheet(isPresented: $showTranslationPicker) {
            TranslationLanguagePickerView(
                selectedLanguage: $selectedTranslationLanguage,
                onSelect: { triggerTranslation() }
            )
        }
        .translationTask(translationConfig) { session in
            Task { @MainActor in
                await performTranslation(using: session)
            }
        }
        .onAppear {
            likedState = player.currentTrack?.liked
            loadLyrics()
        }
        .onChange(of: player.currentTrack?.id) { _, _ in
            likedState = player.currentTrack?.liked
            loadLyrics()
        }
    }

    // MARK: - Background

    private var artworkBackground: some View {
        Group {
            if let artUrl = player.currentTrack?.artUrl, let url = URL(string: artUrl) {
                DynamicArtworkBackground(url: url, move: true)
            } else {
                Color.secondary.opacity(0.5)
            }
        }
        .ignoresSafeArea()
    }

    // MARK: - Artwork + track info

    private func artworkContent(geo: GeometryProxy) -> some View {
        let maxWidth = geo.size.width - 44
        let maxHeight = geo.size.height * 0.50
        let baseSize = min(maxWidth, maxHeight, 420)
        let reduced: CGFloat = 40
        let playingScale = player.isPlaying ? 1.0 : max(0.0, (baseSize - reduced) / baseSize)

        return VStack(spacing: 32) {
            AnimatedArtworkCard(
                url: player.currentTrack?.artUrl.flatMap { URL(string: $0) },
                queueItemId: player.currentTrack?.id.description,
                queueIndex: player.currentIndex,
                size: baseSize,
                cornerRadius: 20,
                playingScale: playingScale
            )

            HStack(spacing: 4) {
                trackInfo(compact: false)
                Spacer()
                Button {
                    toggleLike()
                } label: {
                    Image(systemName: currentLiked ? "heart.fill" : "heart")
                        .font(.title2)
                        .foregroundStyle(currentLiked ? .pink : .white.opacity(0.7))
                }
                .disabled(isTogglingLike)
            }
            .padding(.horizontal, 24)
            .padding(.top, 10)
        }
    }

    // MARK: - Track info

    @ViewBuilder
    private func trackInfo(compact: Bool) -> some View {
        if compact {
            HStack(spacing: 12) {
                ArtworkImage(
                    url: player.currentTrack?.artUrl,
                    size: 56,
                    cornerRadius: 8
                )
                .shadow(color: .black.opacity(0.3), radius: 8, x: 0, y: 4)

                VStack(alignment: .leading, spacing: 4) {
                    MarqueeText(
                        text: player.currentTrack?.name ?? "Nothing Playing",
                        font: UIFont.preferredFont(forTextStyle: .headline),
                        leftFade: 16, rightFade: 16, startDelay: 3
                    )
                    .foregroundStyle(.white)

                    MarqueeText(
                        text: player.currentTrack?.displayArtist ?? "",
                        font: UIFont.preferredFont(forTextStyle: .subheadline),
                        leftFade: 16, rightFade: 16, startDelay: 3
                    )
                    .foregroundStyle(.white.opacity(0.8))
                }

                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
        } else {
            VStack(alignment: .leading, spacing: 4) {
                MarqueeText(
                    text: player.currentTrack?.name ?? "Nothing Playing",
                    font: UIFont.systemFont(ofSize: 24, weight: .bold),
                    leftFade: 16, rightFade: 16, startDelay: 3
                )
                .foregroundStyle(.white)

                MarqueeText(
                    text: player.currentTrack?.displayArtist ?? "",
                    font: UIFont.systemFont(ofSize: 20, weight: .medium),
                    leftFade: 16, rightFade: 16, startDelay: 3
                )
                .foregroundStyle(.white.opacity(0.8))
            }
        }
    }

    // MARK: - Lyrics

    @ViewBuilder
    private var lyricsView: some View {
        if let lyrics = lyrics {
            if let richsync = lyrics.richsync {
                RichLyricsView(
                    richsync: richsync,
                    currentTimeMs: Int(player.currentTime * 1000),
                    onSeek: { ms in player.seek(to: Double(ms) / 1000) },
                    fontDesign: .default,
                    fontWeight: .bold,
                    fontSizeMultiplier: 1.0,
                    fadeCompletedLines: false,
                    translations: translations,
                    bgVoxTranslations: bgVoxTranslations
                )
                .padding(.top, 76)
            } else {
                SyncedLyricsView(
                    lyrics: lyrics,
                    currentTimeMs: Int(player.currentTime * 1000),
                    onSeek: { ms in player.seek(to: Double(ms) / 1000) },
                    translations: translations
                )
                .padding(.top, 76)
            }
        } else {
            VStack {
                Spacer()
                Image(systemName: "music.note")
                    .font(.system(size: 60))
                    .foregroundStyle(.white.opacity(0.5))
                Text("No lyrics available")
                    .font(.headline)
                    .foregroundStyle(.white.opacity(0.7))
                Spacer()
            }
            .padding(.top, 76)
        }
    }

    // MARK: - Controls

    private func controls(compact: Bool) -> some View {
        VStack(spacing: compact ? 32 : 45) {
            progressBar
                .padding(.horizontal, 24)

            HStack(spacing: 48) {
                Button {
                    player.previous()
                } label: {
                    Image(systemName: "backward.fill")
                        .font(.largeTitle)
                        .foregroundStyle(.white)
                }

                Button {
                    player.togglePlayPause()
                } label: {
                    Image(systemName: player.isPlaying ? "pause.fill" : "play.fill")
                        .font(.system(size: 54))
                        .foregroundStyle(.white)
                }

                Button {
                    player.next()
                } label: {
                    Image(systemName: "forward.fill")
                        .font(.largeTitle)
                        .foregroundStyle(.white)
                }
            }
            .padding(.horizontal, 24)

            bottomBar
        }
    }

    // MARK: - Progress bar

    private var progressBar: some View {
        let currentTime = isDraggingProgress ? progressDragValue : player.currentTime
        let duration = max(player.duration, 1)
        let progress = currentTime / duration

        return VStack(spacing: 6) {
            GeometryReader { geo in
                VStack(spacing: 8) {
                    ZStack(alignment: .leading) {
                        RoundedRectangle(cornerRadius: 999)
                            .fill(Color.white.opacity(0.3))
                            .frame(height: 6)

                        RoundedRectangle(cornerRadius: 999)
                            .fill(Color.white)
                            .frame(
                                width: geo.size.width * CGFloat(min(max(progress, 0), 1)), height: 6
                            )
                    }
                    HStack {
                        Text(formatTime(currentTime)).monospaced()
                        Spacer()
                        if player.useHLS, let profile = player.currentHLSProfile {
                            HStack(spacing: 4) {
                                if profile.codec == "flac" {
                                    Text("HI-RES")
                                        .fontWeight(.semibold)
                                        .foregroundStyle(Color.accentColor)
                                    if let track = player.currentTrack,
                                        let sr = track.sampleRate, let bits = track.bitsPerSample
                                    {
                                        Text("·")
                                        Text("\(bits)-bit \(sr / 1000)kHz FLAC")
                                    }
                                } else if let bitrate = profile.bitrate {
                                    Text("\(bitrate / 1000) KBPS \(profile.codec.uppercased())")
                                        .fontWeight(.semibold)
                                }
                            }
                        } else if let track = player.currentTrack, track.lossless == true {
                            HStack(spacing: 4) {
                                Text("HI-RES")
                                    .fontWeight(.semibold)
                                    .foregroundStyle(Color.accentColor)
                                if let sr = track.sampleRate, let bits = track.bitsPerSample {
                                    Text("·")
                                    Text("\(bits)-bit \(sr / 1000)kHz")
                                }
                            }
                        }
                        Spacer()
                        Text(formatTime(duration)).monospaced()
                    }
                    .font(.caption)
                    .monospaced()
                    .foregroundStyle(.white.opacity(0.8))
                }
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(minimumDistance: 0)
                        .onChanged { val in
                            isDraggingProgress = true
                            let pct = val.location.x / geo.size.width
                            progressDragValue = max(0, min(Double(pct) * duration, duration))
                        }
                        .onEnded { _ in
                            player.seek(to: progressDragValue)
                            isDraggingProgress = false
                        }
                )
            }
            .frame(height: 32)
        }
    }

    // MARK: - Bottom bar

    private var bottomBar: some View {
        HStack(spacing: 0) {
            Button {
                withAnimation(.spring(response: 0.4, dampingFraction: 0.8)) {
                    showLyrics.toggle()
                }
            } label: {
                Image(systemName: "quote.bubble")
                    .font(.title2)
                    .foregroundStyle(showLyrics ? Color.accentColor : .white.opacity(0.7))
                    .frame(maxWidth: 52)
                    .padding(.vertical, 12)
                    .background(
                        Circle().fill(showLyrics ? Color.teal.opacity(0.2) : Color.clear)
                    )
            }

            if showLyrics {
                Button {
                    if translations != nil {
                        translations = nil
                        bgVoxTranslations = nil
                    } else {
                        showTranslationPicker = true
                    }
                } label: {
                    Image(
                        systemName: translations != nil
                            ? "character.bubble.fill" : "character.bubble"
                    )
                    .font(.title2)
                    .foregroundStyle(translations != nil ? Color.accentColor : .white.opacity(0.7))
                    .frame(maxWidth: 52)
                    .padding(.vertical, 12)
                    .background(
                        Circle().fill(translations != nil ? Color.teal.opacity(0.2) : Color.clear)
                    )
                }
                .transition(.opacity.combined(with: .scale(scale: 0.8)))
            }

            Spacer()

            Button {
                showQueue = true
            } label: {
                Image(systemName: "list.bullet")
                    .font(.title2)
                    .foregroundStyle(.white.opacity(0.7))
                    .padding(.vertical, 12)
            }
        }
        .padding(.horizontal, 24)
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: showLyrics)
    }

    // MARK: - Helpers

    private var currentLiked: Bool {
        likedState ?? player.currentTrack?.liked ?? false
    }

    private func formatTime(_ seconds: Double) -> String {
        let s = Int(max(0, seconds))
        return String(format: "%d:%02d", s / 60, s % 60)
    }

    private func toggleLike() {
        guard let track = player.currentTrack, !isTogglingLike else { return }
        isTogglingLike = true
        likedState = !(likedState ?? track.liked ?? false)
        Task {
            do {
                let result = try await apiClient.toggleLike(trackId: track.id)
                await MainActor.run { likedState = result.liked }
            } catch {
                await MainActor.run { likedState = track.liked }
            }
            await MainActor.run { isTogglingLike = false }
        }
    }

    private func triggerTranslation() {
        guard lyrics != nil else { return }
        if translationConfig == nil {
            if let langCode = selectedTranslationLanguage {
                translationConfig = TranslationSession.Configuration(
                    source: nil,
                    target: Locale.Language(identifier: langCode)
                )
            } else {
                translationConfig = TranslationSession.Configuration()
            }
        } else {
            translationConfig?.invalidate()
        }
    }

    private func performTranslation(using session: TranslationSession) async {
        guard let lyrics else { return }

        let mainTexts: [String]
        let bgTexts: [String]

        if let richsync = lyrics.richsync {
            let lines = richsync.sections.flatMap(\.lines)
            mainTexts = lines.map(\.text)
            // Parallel array: empty string for lines with no bgVox
            bgTexts = lines.map { $0.bgVox?.text ?? "" }
        } else {
            mainTexts = lyrics.lines.lines.map(\.text)
            bgTexts = []
        }

        // Single batch call: main lines followed by bgVox lines
        let allTexts = mainTexts + bgTexts

        do {
            let allTranslated = try await translationService.translateBatch(
                allTexts, session: session)
            translations = Array(allTranslated.prefix(mainTexts.count))
            if !bgTexts.isEmpty {
                bgVoxTranslations = Array(allTranslated.suffix(bgTexts.count))
            }
        } catch {
            print("[Translation] failed: \(error)")
        }
    }

    private func loadLyrics() {
        guard let track = player.currentTrack else {
            print("[Lyrics] loadLyrics called but no current track")
            return
        }
        print(
            "[Lyrics] fetching for '\(track.name)' by '\(track.displayArtist)' on '\(track.albumName)'"
        )
        lyrics = nil
        translations = nil
        bgVoxTranslations = nil
        Task {
            do {
                let jlf = try await umiClient.fetchLyrics(
                    track: track.name,
                    artist: track.displayArtist,
                    album: track.albumName
                )
                print(
                    "[Lyrics] success, \(jlf.lines.lines.count) lines from source '\(jlf.source)'")
                await MainActor.run { self.lyrics = jlf }
            } catch {
                print("[Lyrics] fetch failed: \(error)")
                await MainActor.run { self.lyrics = nil }
            }
        }
    }
}

#Preview {
    NowPlayingSheetView(player: .preview, dismiss: {})
}

// MARK: - Queue sheet

private struct NowPlayingQueueSheet: View {
    @Bindable var player: PlayerEngine
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List(Array(player.queue.enumerated()), id: \.element.id) { index, track in
                HStack(spacing: 12) {
                    ArtworkImage(url: track.artUrl, size: 44, cornerRadius: 6)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(track.name)
                            .font(.subheadline)
                            .lineLimit(1)
                            .foregroundStyle(
                                index == player.currentIndex ? Color.accentColor : .primary)
                        Text(track.displayArtist)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }

                    Spacer()

                    if index == player.currentIndex {
                        Image(systemName: "waveform")
                            .foregroundStyle(.tint)
                            .font(.caption)
                    }
                }
                .padding(.vertical, 2)
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
